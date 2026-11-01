//! Parsing of the attribute arguments.
//!
//! The attribute accepts one positional base path plus optional
//! comma-separated options:
//!
//! ```text
//! #[localizedir("templates/localized", fallback = "en", parent = "auth", case = "kebab")]
//! ```
//!
//! Parsing is a plain [`Parse`] implementation over `proc_macro2` tokens, so
//! it is fully unit-testable without a compiler context.

use syn::parse::{Parse, ParseStream};

use syn::{Ident, LitStr, Token};

/// Parsed `#[localizedir(...)]` arguments.
#[derive(Debug)]
pub(crate) struct Args {
    /// The positional base path, exactly as written (kept for `base_path()`
    /// introspection and passed verbatim to `include_dir!`).
    pub(crate) base_path: LitStr,
    /// `fallback = "..."` — locale used when the requested one is missing.
    pub(crate) fallback: Option<LitStr>,
    /// `parent = "..."` — overrides inference from the enum name.
    pub(crate) parent: Option<LitStr>,
    /// `case = "..."` — variant directory naming scheme (`snake` default).
    pub(crate) case: Option<LitStr>,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let base_path: LitStr = input.parse()?;
        let mut args = Self {
            base_path,
            fallback: None,
            parent: None,
            case: None,
        };

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break; // tolerate a trailing comma
            }

            let key: Ident = input.parse()?;
            let _equals = input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;

            match &*key.to_string() {
                "fallback" => set_once(&mut args.fallback, value, &key)?,
                "parent" => set_once(&mut args.parent, value, &key)?,
                "case" => set_once(&mut args.case, value, &key)?,
                unknown => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown option `{unknown}`; expected one of `fallback`, `parent`, `case`"
                        ),
                    ));
                }
            }
        }

        Ok(args)
    }
}

/// Stores `value` into `slot`, or errors with a duplicate-option message.
fn set_once(slot: &mut Option<LitStr>, value: LitStr, key: &Ident) -> syn::Result<()> {
    if slot.is_some() {
        return Err(syn::Error::new(
            key.span(),
            format!("duplicate option `{key}`"),
        ));
    }
    *slot = Some(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use syn::parse_str;

    use super::Args;

    fn parse(attr: &str) -> syn::Result<Args> {
        parse_str::<Args>(attr)
    }

    #[test]
    fn parses_positional_base_path_only() {
        let args = parse(r#""templates/localized""#).expect("parses");
        assert_eq!(args.base_path.value(), "templates/localized");
        assert!(args.fallback.is_none());
        assert!(args.parent.is_none());
        assert!(args.case.is_none());
    }

    #[test]
    fn parses_every_option() {
        let args = parse(r#""locales", fallback = "en", parent = "auth", case = "kebab""#)
            .expect("parses");
        assert_eq!(args.base_path.value(), "locales");
        assert_eq!(args.fallback.as_ref().expect("fallback").value(), "en");
        assert_eq!(args.parent.as_ref().expect("parent").value(), "auth");
        assert_eq!(args.case.as_ref().expect("case").value(), "kebab");
    }

    #[test]
    fn tolerates_trailing_comma() {
        let args = parse(r#""locales","#).expect("parses");
        assert_eq!(args.base_path.value(), "locales");
    }

    #[test]
    fn rejects_missing_base_path() {
        let error = parse("").expect_err("empty input must fail");
        assert!(error.to_string().contains("expected"), "{error}");
    }

    #[test]
    fn rejects_non_literal_base_path() {
        let error = parse("locales_dir").expect_err("identifier is not a literal");
        assert!(error.to_string().contains("string literal"), "{error}");
    }

    #[test]
    fn rejects_unknown_option() {
        let error = parse(r#""locales", fallbackx = "en""#).expect_err("unknown key");
        assert!(
            error.to_string().contains("unknown option `fallbackx`"),
            "{error}"
        );
    }

    #[test]
    fn rejects_duplicate_option() {
        let error =
            parse(r#""locales", fallback = "en", fallback = "fr""#).expect_err("duplicate key");
        assert!(
            error.to_string().contains("duplicate option `fallback`"),
            "{error}"
        );
    }

    #[test]
    fn rejects_non_literal_option_value() {
        let error = parse(r#""locales", fallback = en"#).expect_err("bare ident value");
        assert!(error.to_string().contains("string literal"), "{error}");
    }
}
