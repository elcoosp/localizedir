//! Compile-time localized directory inclusion for Rust.
//!
//! [`localizedir`] turns an enum of unit variants into locale-aware
//! accessors over directories embedded with [`include_dir`]. The whole base
//! directory is embedded **once** at compile time; every lookup is a cheap
//! navigation inside that embedded tree.
//!
//! # Quick start
//!
//! Given this directory layout next to your crate manifest:
//!
//! ```text
//! templates/localized/
//! ├── en/auth/welcome/note.txt
//! ├── fr/auth/welcome/note.txt
//! └── de/auth/welcome/note.txt
//! ```
//!
//! the attribute macro generates a [`LocalizeDir`] implementation:
//!
//! ```rust
//! use localizedir::{LocalizeDir, localizedir};
//!
//! #[localizedir("tests/fixtures/localized")]
//! enum AuthDirs {
//!     Welcome,
//!     ForgotPassword,
//! }
//!
//! // Non-panicking lookup returns the embedded directory for a locale.
//! let dir = AuthDirs::Welcome.try_to_localized_dir("fr").expect("fr exists");
//! assert!(dir.get_file(AuthDirs::Welcome.full_path("fr").join("note.txt")).is_some());
//!
//! // Panicking lookup, plus compile-time metadata:
//! let dir = AuthDirs::Welcome.to_localized_dir("en");
//! assert_eq!(AuthDirs::available_locales(), &["de", "en", "fr"]);
//! assert_eq!(AuthDirs::parent_folder(), "auth");
//! assert_eq!(AuthDirs::base_path(), "tests/fixtures/localized");
//! assert_eq!(AuthDirs::Welcome.dir_name(), "welcome");
//! assert!(AuthDirs::has_locale("de"));
//! assert_eq!(
//!     AuthDirs::ForgotPassword.full_path("fr"),
//!     std::path::PathBuf::from("fr/auth/forgot_password"),
//! );
//! ```
//!
//! # Attribute options
//!
//! | Option | Default | Meaning |
//! |--------|---------|---------|
//! | `"<base path>"` | — | Directory containing one subdirectory per locale. |
//! | `fallback = "<locale>"` | none | Locale used by [`LocalizeDir::to_localized_dir`] when the requested locale has no directory. |
//! | `parent = "<folder>"` | inferred | Overrides the parent folder inferred from the enum name. |
//! | `case = "snake" \| "kebab" \| "flat"` | `"snake"` | Naming scheme for variant directories. |
//!
//! Variants may additionally carry `#[dir_name = "..."]` to override their
//! individual directory name.
//!
//! # Naming conventions
//!
//! The parent folder is inferred from the enum name by stripping a known
//! suffix (`Dirs`, `Dir`, `Templates`, `Template`, `Documents`, `Document`,
//! `Directories`, `Docs`, `Assets`, `Asset`, `Files`, `File`, `Locales`,
//! `Locale`) and converting the remainder to `snake_case`
//! (`AuthTemplates` → `auth`, `APIResponseDirs` → `api_response`).
//!
//! # Compile-time guarantees
//!
//! The macro fails to compile — with a message pointing at the offending
//! token — when the base path is missing, when a `(locale, variant)`
//! directory is missing, when two variants map to the same directory name,
//! or when the attribute is malformed. Locales are discovered from the
//! filesystem at compile time and are always sorted.
//!
//! # Dependencies
//!
//! Downstream crates need both `localizedir` and `include_dir` (the macro
//! expands to `include_dir::include_dir!`, which resolves in *your* crate):
//!
//! ```toml
//! [dependencies]
//! localizedir = "0.2"
//! include_dir = "0.7"
//! ```

pub use include_dir::Dir;
pub use localizedir_macros::localizedir;

/// Locale-aware access over directories embedded by [`localizedir`].
///
/// Implemented for every enum annotated with `#[localizedir(...)]`.
pub trait LocalizeDir {
    /// Locales discovered at compile time under the base path, sorted.
    fn available_locales() -> &'static [&'static str]
    where
        Self: Sized;

    /// The base path exactly as written in the attribute (before `$VAR`
    /// interpolation).
    fn base_path() -> &'static str
    where
        Self: Sized;

    /// Parent folder between the locale directory and the variant
    /// directories; inferred from the enum name unless overridden with
    /// `parent = "..."`.
    fn parent_folder() -> &'static str
    where
        Self: Sized;

    /// Locale used as a last resort by [`LocalizeDir::to_localized_dir`]
    /// when configured via `fallback = "..."`; `None` by default.
    #[must_use]
    fn fallback_locale() -> Option<&'static str>
    where
        Self: Sized,
    {
        None
    }

    /// Directory name of this variant, in the configured naming scheme
    /// (`case = "..."`) or the `#[dir_name = "..."]` override.
    fn dir_name(&self) -> &'static str;

    /// The embedded path of this variant's directory under `locale`,
    /// relative to the embedded root.
    ///
    /// This is a pure composition and does **not** require the locale to
    /// exist; for discovered locales it always equals [`Dir::path`] of the
    /// directory returned by the lookup methods.
    ///
    /// [`Dir::path`]: include_dir::Dir::path
    fn full_path(&self, locale: &str) -> std::path::PathBuf;

    /// Returns the embedded directory for `locale`, or `None` when the
    /// locale is unknown.
    ///
    /// This is the non-panicking primitive; [`LocalizeDir::to_localized_dir`]
    /// layers fallback and panicking semantics on top of it.
    fn try_to_localized_dir(&self, locale: &str) -> Option<&'static Dir<'static>>;

    /// Returns the embedded directory for `locale`.
    ///
    /// When a `fallback = "<locale>"` option was configured, an unknown
    /// locale falls back to the fallback locale's directory.
    ///
    /// # Panics
    ///
    /// Panics when neither `locale` (nor the configured fallback, if any)
    /// has an embedded directory for this variant. Use
    /// [`LocalizeDir::try_to_localized_dir`] for a non-panicking lookup.
    fn to_localized_dir(&self, locale: &str) -> &'static Dir<'static> {
        self.try_to_localized_dir(locale).unwrap_or_else(|| {
            panic!(
                "localizedir: no embedded directory for locale `{locale}` (variant `{}`)",
                self.dir_name()
            )
        })
    }

    /// `true` when `locale` was discovered at compile time.
    #[must_use]
    fn has_locale(locale: &str) -> bool
    where
        Self: Sized,
    {
        Self::available_locales().contains(&locale)
    }
}
