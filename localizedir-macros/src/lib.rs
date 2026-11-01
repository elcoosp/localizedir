//! Procedural macro implementation of `#[localizedir]`.
//!
//! You should depend on the [`localizedir`](https://docs.rs/localizedir)
//! facade crate instead; this crate exists only to keep the heavyweight
//! `syn` dependency out of downstream builds that do not use the macro.
//!
//! Module layout (private implementation details):
//! - `args`: attribute argument parsing
//! - `naming`: enum/variant to directory-name rules
//! - `discovery`: compile-time filesystem access
//! - `expand`: validation and code generation

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod args;
mod discovery;
mod expand;
mod naming;

/// Generates locale-aware accessors for an enum of embedded directories.
///
/// The attribute expects one positional base path (resolved relative to the
/// crate manifest directory, exactly like `include_dir!`, including `$VAR`
/// interpolation) plus optional options:
///
/// ```text
/// #[localizedir("templates/localized", fallback = "en", parent = "auth", case = "kebab")]
/// enum AuthDirs {
///     #[dir_name = "welcome_v2"]
///     Welcome,
///     ForgotPassword,
/// }
/// ```
///
/// See the `localizedir` crate documentation for the generated API and a
/// runnable example; the example lives there because it needs the compiled
/// fixture tree.
#[proc_macro_attribute]
pub fn localizedir(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::DeriveInput);
    let parsed_args = parse_macro_input!(attr as args::Args);
    expand::expand(&input, &parsed_args)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
