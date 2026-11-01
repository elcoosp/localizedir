//! Code generation: validates the macro input against the filesystem and
//! emits the inherent + trait impl blocks.
//!
//! `expand` is a pure `(DeriveInput, Args) -> syn::Result<TokenStream2>`
//! function; the proc-macro entry point in `lib.rs` is a thin facade over
//! it. Everything here is unit-testable without a compiler context.

use convert_case::Case;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Expr, Fields, Lit, LitStr, Meta, Variant};

use crate::args::Args;
use crate::discovery;
use crate::naming;

/// Expands `#[localizedir(...)]` into the enum plus its inherent and
/// [`LocalizeDir`](crate::LocalizeDir) impl blocks.
///
/// `LOCALIZEDIR_BASE` embeds the whole base directory **once** via
/// `include_dir!`, anchored at `$CARGO_MANIFEST_DIR` for relative inputs;
/// lookups navigate that tree with root-relative paths.
pub(crate) fn expand(input: &DeriveInput, args: &Args) -> syn::Result<TokenStream> {
    let enum_name = &input.ident;
    let data_enum = validate_input(input)?;
    let variants: Vec<&Variant> = data_enum.variants.iter().collect();

    let case = parse_case_checked(args.case.as_ref())?;
    let parent_folder = parent_folder_of(input, args.parent.as_ref())?;
    let dir_names = dir_names_of(enum_name, &variants, case)?;
    let anchored = anchor_base_path(args)?;
    let locales = validate_filesystem(&anchored, &parent_folder, &dir_names)
        .map_err(|error| syn::Error::new_spanned(&args.base_path, error))?;

    let variant_idents: Vec<_> = variants.iter().map(|variant| &variant.ident).collect();
    let dir_name_lits: Vec<LitStr> = dir_names
        .iter()
        .zip(&variant_idents)
        .map(|(name, ident)| LitStr::new(name, ident.span()))
        .collect();
    let locale_lits: Vec<&str> = locales.iter().map(String::as_str).collect();

    let fallback_locale = args.fallback.as_ref().map(|lit| {
        quote! {
            fn fallback_locale() -> Option<&'static str> {
                Some(#lit)
            }
        }
    });
    let fallback_override = args.fallback.as_ref().map(|lit| {
        quote! {
            fn to_localized_dir(&self, locale: &str) -> &'static ::include_dir::Dir<'static> {
                const FALLBACK: &str = #lit;
                match self.try_to_localized_dir(locale) {
                    Some(dir) => dir,
                    None => self.try_to_localized_dir(FALLBACK).unwrap_or_else(|| {
                        panic!(
                            "localizedir: no embedded directory for variant `{}` with locale `{locale}` nor fallback `{FALLBACK}` (base: `{}`)",
                            self.dir_name(),
                            <Self as ::localizedir::LocalizeDir>::base_path(),
                        )
                    }),
                }
            }
        }
    });

    let base_path_lit = &args.base_path;
    let include_arg = LitStr::new(&anchored.include_arg, args.base_path.span());
    let enum_item = reemit_enum_without_dir_name_attrs(input, data_enum);

    Ok(quote! {
        #enum_item

        #[doc(hidden)]
        impl #enum_name {
            /// The whole base directory, embedded exactly once.
            const LOCALIZEDIR_BASE: ::include_dir::Dir<'static> =
                ::include_dir::include_dir!(#include_arg);

            /// Composes the embedded path of this variant under `locale`
            /// (relative to the embedded root, matching `Dir::path()`).
            fn localizedir_path(&self, locale: &str) -> ::std::path::PathBuf {
                ::std::path::Path::new(locale)
                    .join(Self::parent_folder())
                    .join(self.dir_name())
            }
        }

        impl ::localizedir::LocalizeDir for #enum_name {
            fn available_locales() -> &'static [&'static str] {
                &[#(#locale_lits),*]
            }

            fn base_path() -> &'static str {
                #base_path_lit
            }

            fn parent_folder() -> &'static str {
                #parent_folder
            }

            #fallback_locale

            fn dir_name(&self) -> &'static str {
                match self {
                    #(Self::#variant_idents => #dir_name_lits,)*
                }
            }

            fn full_path(&self, locale: &str) -> ::std::path::PathBuf {
                self.localizedir_path(locale)
            }

            fn try_to_localized_dir(&self, locale: &str) -> Option<&'static ::include_dir::Dir<'static>> {
                Self::LOCALIZEDIR_BASE.get_dir(self.localizedir_path(locale))
            }

            #fallback_override
        }
    })
}

/// Validates the macro target: an enum, non-generic, with unit variants.
fn validate_input(input: &DeriveInput) -> syn::Result<&syn::DataEnum> {
    let Data::Enum(data_enum) = &input.data else {
        return Err(syn::Error::new_spanned(
            input,
            "`localizedir` can only be applied to enums",
        ));
    };
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "`localizedir` does not support generic enums; the embedded directories are concrete `include_dir` trees",
        ));
    }
    if data_enum.variants.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "`localizedir` requires at least one variant",
        ));
    }
    for variant in &data_enum.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                variant,
                "`localizedir` only supports unit variants (no fields)",
            ));
        }
    }
    Ok(data_enum)
}

/// Resolves the parent folder from the `parent = "..."` override or the
/// enum name, rejecting empty results.
fn parent_folder_of(input: &DeriveInput, parent: Option<&LitStr>) -> syn::Result<String> {
    let folder = if let Some(lit) = parent {
        lit.value()
    } else {
        naming::infer_parent_folder(&input.ident.to_string())
    };
    if folder.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "inferred parent folder is empty; rename the enum or pass `parent = \"...\"`",
        ));
    }
    Ok(folder)
}

/// Resolves the per-variant directory names (override or `case`), rejecting
/// empty names and collisions.
fn dir_names_of(
    enum_name: &syn::Ident,
    variants: &[&Variant],
    case: Case<'static>,
) -> syn::Result<Vec<String>> {
    let mut dir_names = Vec::with_capacity(variants.len());
    for variant in variants {
        let name = if let Some(overridden) = variant_dir_override(variant)? {
            overridden
        } else {
            naming::variant_dir_name(&raw_ident_name(&variant.ident), case)
        };
        if name.is_empty() {
            return Err(syn::Error::new_spanned(
                variant,
                "directory name is empty; pass `#[dir_name = \"...\"]` with a non-empty name",
            ));
        }
        dir_names.push(name);
    }

    let duplicates = find_duplicates(&dir_names);
    if !duplicates.is_empty() {
        return Err(syn::Error::new_spanned(
            enum_name,
            format!(
                "variants map to duplicate directory names: {}",
                duplicates.join(", ")
            ),
        ));
    }
    Ok(dir_names)
}

/// Anchors the base path at the package manifest directory.
fn anchor_base_path(args: &Args) -> syn::Result<discovery::AnchoredPath> {
    discovery::anchor_path(
        &args.base_path.value(),
        std::env::var("CARGO_MANIFEST_DIR").ok().as_deref(),
    )
    .map_err(|error| syn::Error::new_spanned(&args.base_path, error))
}

/// Validates the filesystem: locales discovered and every
/// `(locale, variant)` directory present.
fn validate_filesystem(
    anchored: &discovery::AnchoredPath,
    parent_folder: &str,
    dir_names: &[String],
) -> Result<Vec<String>, String> {
    let base_path = std::path::Path::new(&anchored.expanded);
    let locales = discovery::discover_locales(base_path)?;
    let missing = discovery::missing_variant_dirs(base_path, &locales, parent_folder, dir_names);
    if missing.is_empty() {
        return Ok(locales);
    }
    let listed: Vec<String> = missing
        .iter()
        .map(|path| format!("  - {}", path.display()))
        .collect();
    Err(format!(
        "missing directories for some (locale, variant) pairs:\n{}",
        listed.join("\n")
    ))
}

/// Re-emits the enum, dropping the `#[dir_name = "..."]` attributes the
/// macro has already consumed.
///
/// Helper attributes of `#[proc_macro_attribute]` macros cannot be declared
/// the way derive macros declare them, so an unknown attribute would fail
/// resolution if re-emitted verbatim.
fn reemit_enum_without_dir_name_attrs(
    input: &DeriveInput,
    data_enum: &syn::DataEnum,
) -> TokenStream {
    let attrs = &input.attrs;
    let vis = &input.vis;
    let name = &input.ident;
    let variants = data_enum.variants.iter().map(|variant| {
        let kept_attrs: Vec<_> = variant
            .attrs
            .iter()
            .filter(|attr| !attr.path().is_ident("dir_name"))
            .collect();
        let ident = &variant.ident;
        let discriminant = variant
            .discriminant
            .as_ref()
            .map(|(_, expr)| quote! { = #expr });
        quote! { #(#kept_attrs)* #ident #discriminant }
    });
    quote! {
        #(#attrs)*
        #vis enum #name {
            #(#variants,)*
        }
    }
}

/// Maps the `case = "..."` value onto a [`Case`], erroring on unknown names.
fn parse_case_checked(case: Option<&LitStr>) -> syn::Result<Case<'static>> {
    let Some(lit) = case else {
        return Ok(Case::Snake);
    };
    let case_name = lit.value();
    if let Some(case) = naming::parse_case(&case_name) {
        return Ok(case);
    }
    let known: Vec<&str> = naming::KNOWN_CASES.iter().map(|(name, _)| *name).collect();
    Err(syn::Error::new(
        lit.span(),
        format!(
            "unknown case `{case_name}`; expected one of {}",
            known.join(", ")
        ),
    ))
}

/// Extracts a `#[dir_name = "..."]` override from a variant, if present.
fn variant_dir_override(variant: &Variant) -> syn::Result<Option<String>> {
    let mut overridden = None;
    for attr in &variant.attrs {
        if !attr.path().is_ident("dir_name") {
            continue;
        }
        let Meta::NameValue(name_value) = &attr.meta else {
            return Err(syn::Error::new_spanned(
                attr,
                "expected `#[dir_name = \"...\"]`",
            ));
        };
        let Expr::Lit(expr_lit) = &name_value.value else {
            return Err(syn::Error::new_spanned(
                &name_value.value,
                "`dir_name` must be a string literal",
            ));
        };
        let Lit::Str(lit) = &expr_lit.lit else {
            return Err(syn::Error::new_spanned(
                expr_lit,
                "`dir_name` must be a string literal",
            ));
        };
        if overridden.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "duplicate `#[dir_name]` attribute on variant",
            ));
        }
        overridden = Some(lit.value());
    }
    Ok(overridden)
}

/// Returns `ident.to_string()` without a raw-identifier `r#` prefix.
fn raw_ident_name(ident: &syn::Ident) -> String {
    let name = ident.to_string();
    match name.strip_prefix("r#") {
        Some(stripped) => stripped.to_owned(),
        None => name,
    }
}

/// Returns values occurring more than once, sorted, without repetitions.
fn find_duplicates(values: &[String]) -> Vec<String> {
    let mut sorted = values.to_vec();
    sorted.sort();
    let mut duplicates: Vec<String> = sorted
        .windows(2)
        .filter(|pair| pair[0] == pair[1])
        .map(|pair| pair[0].clone())
        .collect();
    duplicates.dedup();
    duplicates
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use syn::parse_str;

    use super::expand;
    use crate::args::Args;

    /// Builds a plausible locale tree under a fresh temp directory.
    fn make_tree(base: &Path, locales: &[&str], parent: &str, dirs: &[&str]) {
        for locale in locales {
            for dir in dirs {
                let dir = base.join(locale).join(parent).join(dir);
                fs::create_dir_all(&dir).expect("create dir");
                fs::write(dir.join("note.txt"), format!("{locale}:{}", dir.display()))
                    .expect("write file");
            }
        }
    }

    fn expand_with(attr: &str, item: &str) -> Result<String, String> {
        let input: syn::DeriveInput = parse_str(item).map_err(|error| format!("parse: {error}"))?;
        let args: Args = parse_str(attr).map_err(|error| format!("args: {error}"))?;
        expand(&input, &args)
            .map(|tokens| tokens.to_string())
            .map_err(|error| error.to_string())
    }

    fn temp_base(attr_template: &str, base: &Path) -> String {
        attr_template.replace("{{BASE}}", &base.to_string_lossy())
    }

    #[test]
    fn happy_path_generates_impls() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let base = temp.path().join("locales");
        make_tree(
            &base,
            &["de", "en", "fr"],
            "auth",
            &["welcome", "forgot_password"],
        );

        let output = expand_with(
            &temp_base(r#""{{BASE}}""#, temp.path().join("locales").as_path()),
            "enum AuthDirs { Welcome, ForgotPassword }",
        )
        .expect("expands");

        for expected in [
            "LOCALIZEDIR_BASE",
            "include_dir",
            "enum AuthDirs",
            "Welcome",
            "ForgotPassword",
            "forgot_password",
            "auth",
            "\"de\"",
            "\"en\"",
            "\"fr\"",
            "available_locales",
            "try_to_localized_dir",
            "full_path",
        ] {
            assert!(
                output.contains(expected),
                "missing `{expected}` in:\n{output}"
            );
        }
        // No fallback configured: the panicking override must be absent.
        assert!(!output.contains("nor fallback"), "{output}");
    }

    #[test]
    fn fallback_generates_override() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let base = temp.path().join("locales");
        make_tree(&base, &["en", "fr"], "email", &["newsletter"]);

        let attr = temp_base(r#""{{BASE}}", fallback = "en""#, &base);
        let output = expand_with(&attr, "enum EmailDirs { Newsletter }").expect("expands");
        assert!(output.contains("FALLBACK"), "{output}");
        assert!(output.contains("Some (\"en\")"), "{output}");
    }

    #[test]
    fn parent_and_case_overrides_apply() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let base = temp.path().join("locales");
        make_tree(&base, &["en"], "legal", &["terms-of-service"]);

        let attr = temp_base(r#""{{BASE}}", parent = "legal", case = "kebab""#, &base);
        let output = expand_with(&attr, "enum WhateverDirs { TermsOfService }").expect("expands");
        assert!(output.contains("terms-of-service"), "{output}");
        assert!(output.contains("\"legal\""), "{output}");
    }

    #[test]
    fn dir_name_attribute_overrides_variant_name() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let base = temp.path().join("locales");
        make_tree(&base, &["en"], "auth", &["welcome_v2"]);

        let attr = temp_base(r#""{{BASE}}""#, &base);
        // The doc comment on the variant is an unrelated attribute that must
        // be ignored (and preserved).
        let item = "enum AuthDirs { /// Documentation.\n #[dir_name = \"welcome_v2\"] Welcome }";
        let output = expand_with(&attr, item).expect("expands");
        assert!(output.contains("welcome_v2"), "{output}");
        assert!(output.contains("Documentation"), "{output}");
        assert!(!output.contains("# [dir_name"), "{output}");
    }

    #[test]
    fn rejects_non_literal_expression_dir_name() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let base = temp.path().join("locales");
        make_tree(&base, &["en"], "auth", &["welcome"]);
        let attr = temp_base(r#""{{BASE}}""#, &base);
        let error = expand_with(&attr, "enum AuthDirs { #[dir_name = 1 + 2] Welcome }")
            .expect_err("must fail");
        assert!(error.contains("string literal"), "{error}");
    }

    #[test]
    fn rejects_non_enum_input() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let attr = temp_base(r#""{{BASE}}""#, temp.path());
        let error = expand_with(&attr, "struct AuthDirs;").expect_err("must fail");
        assert!(error.contains("can only be applied to enums"), "{error}");
    }

    #[test]
    fn rejects_generic_enum() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let attr = temp_base(r#""{{BASE}}""#, temp.path());
        let error = expand_with(&attr, "enum AuthDirs<T> { Welcome(T) }").expect_err("must fail");
        assert!(error.contains("does not support generic enums"), "{error}");
    }

    #[test]
    fn rejects_empty_enum() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let attr = temp_base(r#""{{BASE}}""#, temp.path());
        let error = expand_with(&attr, "enum AuthDirs {}").expect_err("must fail");
        assert!(error.contains("at least one variant"), "{error}");
    }

    #[test]
    fn rejects_fielded_variant() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let base = temp.path().join("locales");
        make_tree(&base, &["en"], "auth", &["welcome"]);
        let attr = temp_base(r#""{{BASE}}""#, temp.path());
        let error = expand_with(
            &attr,
            "enum AuthDirs { Welcome { language: &'static str } }",
        )
        .expect_err("must fail");
        assert!(error.contains("only supports unit variants"), "{error}");
    }

    #[test]
    fn rejects_missing_base_directory() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let error = expand_with(
            &temp_base(r#""{{BASE}}""#, &temp.path().join("nope")),
            "enum AuthDirs { Welcome }",
        )
        .expect_err("must fail");
        assert!(error.contains("does not exist"), "{error}");
    }

    #[test]
    fn rejects_missing_variant_directory() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let base = temp.path().join("locales");
        make_tree(&base, &["en", "fr"], "auth", &["welcome"]);
        let attr = temp_base(r#""{{BASE}}""#, temp.path());
        let error =
            expand_with(&attr, "enum AuthDirs { Welcome, Missing }").expect_err("must fail");
        assert!(error.contains("missing directories"), "{error}");
        assert!(error.contains("auth/missing"), "{error}");
    }

    #[test]
    fn rejects_duplicate_directory_names() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let base = temp.path().join("locales");
        make_tree(&base, &["en"], "auth", &["welcome"]);
        let attr = temp_base(r#""{{BASE}}""#, temp.path());
        let item =
            "enum AuthDirs { #[dir_name = \"welcome\"] Welcome, #[dir_name = \"welcome\"] Hello }";
        let error = expand_with(&attr, item).expect_err("must fail");
        assert!(error.contains("duplicate directory names"), "{error}");
        assert!(error.contains("welcome"), "{error}");
    }

    #[test]
    fn rejects_bad_case_value() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let base = temp.path().join("locales");
        make_tree(&base, &["en"], "auth", &["welcome"]);
        let attr = temp_base(r#""{{BASE}}", case = "camel""#, temp.path());
        let error = expand_with(&attr, "enum AuthDirs { Welcome }").expect_err("must fail");
        assert!(error.contains("unknown case `camel`"), "{error}");
    }

    #[test]
    fn rejects_empty_parent_override() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let base = temp.path().join("locales");
        make_tree(&base, &["en"], "auth", &["welcome"]);
        let attr = temp_base(r#""{{BASE}}", parent = """#, temp.path());
        let error = expand_with(&attr, "enum AuthDirs { Welcome }").expect_err("must fail");
        assert!(error.contains("parent folder is empty"), "{error}");
    }

    #[test]
    fn rejects_empty_dir_name_override() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let base = temp.path().join("locales");
        make_tree(&base, &["en"], "auth", &["welcome"]);
        let attr = temp_base(r#""{{BASE}}""#, temp.path());
        let item = "enum AuthDirs { #[dir_name = \"\"] Welcome }";
        let error = expand_with(&attr, item).expect_err("must fail");
        assert!(error.contains("directory name is empty"), "{error}");
    }

    #[test]
    fn rejects_malformed_dir_name_attribute() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let base = temp.path().join("locales");
        make_tree(&base, &["en"], "auth", &["welcome"]);

        let attr = temp_base(r#""{{BASE}}""#, temp.path());
        let error =
            expand_with(&attr, "enum AuthDirs { #[dir_name] Welcome }").expect_err("must fail");
        assert!(
            error.contains("expected `#[dir_name = \"...\"]`"),
            "{error}"
        );

        let error =
            expand_with(&attr, "enum AuthDirs { #[dir_name = 3] Welcome }").expect_err("must fail");
        assert!(error.contains("string literal"), "{error}");
    }

    #[test]
    fn rejects_duplicate_dir_name_attributes() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let base = temp.path().join("locales");
        make_tree(&base, &["en"], "auth", &["welcome"]);
        let attr = temp_base(r#""{{BASE}}""#, temp.path());
        let item = "enum AuthDirs { #[dir_name = \"a\"] #[dir_name = \"b\"] Welcome }";
        let error = expand_with(&attr, item).expect_err("must fail");
        assert!(error.contains("duplicate `#[dir_name]`"), "{error}");
    }

    #[test]
    fn rejects_unresolvable_env_variable() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let attr = temp_base(
            r#""{{BASE}}""#,
            &temp
                .path()
                .join("$LOCALIZEDIR_DEFINITELY_UNSET_VAR/locales"),
        );
        let error = expand_with(&attr, "enum AuthDirs { Welcome }").expect_err("must fail");
        assert!(error.contains("is not set"), "{error}");
    }

    #[test]
    fn handles_raw_identifier_variants() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let base = temp.path().join("locales");
        make_tree(&base, &["en"], "auth", &["type"]);
        let attr = temp_base(r#""{{BASE}}""#, &base);
        let output = expand_with(&attr, "enum AuthDirs { r#Type }").expect("expands");
        assert!(output.contains("\"type\""), "{output}");
    }
}
