//! Naming rules: enum/variant names to directory names, and the inference of
//! the per-enum parent folder.
//!
//! All conversions go through [`convert_case`], which handles acronym-bearing
//! names (`APIResponse` → `api_response`) far better than a naive
//! insert-underscore-between-case-changes pass.

use convert_case::{Case, Casing};

/// Suffixes stripped from an enum name before inferring the parent folder.
///
/// A name can never end with two of these suffixes at once (they differ in
/// their final character), so scan order is not load-bearing; longest-first
/// merely keeps the table readable.
const STRIPPED_SUFFIXES: [&str; 14] = [
    "Directories",
    "Templates",
    "Documents",
    "Locales",
    "Template",
    "Document",
    "Locale",
    "Dirs",
    "Docs",
    "Dir",
    "Assets",
    "Files",
    "Asset",
    "File",
];

/// Directory-naming schemes accepted by the `case = "..."` attribute option.
///
/// `Case` carries a lifetime for the `Custom` variant; the named schemes
/// used here are all lifetime-free, so they are stored as `Case<'static>`.
pub(crate) const KNOWN_CASES: [(&str, Case<'static>); 3] = [
    ("snake", Case::Snake),
    ("kebab", Case::Kebab),
    ("flat", Case::Flat),
];

/// Infers the parent folder from an enum name.
///
/// The longest known suffix ([`STRIPPED_SUFFIXES`]) is stripped, then the
/// remainder is converted to `snake_case`. If stripping would leave nothing
/// (e.g. the enum is literally called `Dirs`), the full name is converted
/// instead.
///
/// # Examples
///
/// ```text
/// AuthDirs          -> auth
/// EmailTemplates    -> email
/// LegalDocs         -> legal
/// UserProfileDirs   -> user_profile
/// APIResponseDirs   -> api_response
/// Dirs              -> dirs
/// ```
pub(crate) fn infer_parent_folder(enum_name: &str) -> String {
    let stripped = STRIPPED_SUFFIXES
        .iter()
        .find(|suffix| enum_name.len() > suffix.len() && enum_name.ends_with(**suffix))
        .map_or(enum_name, |suffix| {
            &enum_name[..enum_name.len() - suffix.len()]
        });
    stripped.to_case(Case::Snake)
}

/// Converts a variant name into a directory name using `case`.
pub(crate) fn variant_dir_name(variant_name: &str, case: Case<'static>) -> String {
    variant_name.to_case(case)
}

/// Maps the `case = "..."` attribute value onto a [`convert_case::Case`].
pub(crate) fn parse_case(value: &str) -> Option<Case<'static>> {
    KNOWN_CASES
        .iter()
        .find_map(|(name, case)| (*name == value).then_some(*case))
}

#[cfg(test)]
mod tests {
    use super::{KNOWN_CASES, infer_parent_folder, parse_case, variant_dir_name};
    use convert_case::Case;

    #[test]
    fn parent_folder_strips_known_suffixes() {
        assert_eq!(infer_parent_folder("AuthDirs"), "auth");
        assert_eq!(infer_parent_folder("EmailDir"), "email");
        assert_eq!(infer_parent_folder("EmailTemplates"), "email");
        assert_eq!(infer_parent_folder("LegalDocs"), "legal");
        assert_eq!(infer_parent_folder("InvoiceDocuments"), "invoice");
        assert_eq!(infer_parent_folder("UserDirectories"), "user");
        assert_eq!(infer_parent_folder("UserProfileDirs"), "user_profile");
        assert_eq!(infer_parent_folder("BrandAssets"), "brand");
        assert_eq!(infer_parent_folder("StaticFiles"), "static");
        assert_eq!(infer_parent_folder("AppLocales"), "app");
    }

    #[test]
    fn parent_folder_prefers_longest_suffix() {
        // `Templates` must win over `Template`.
        assert_eq!(infer_parent_folder("XTemplates"), "x");
        // `Directories` must win over `Dir`.
        assert_eq!(infer_parent_folder("XDirectories"), "x");
    }

    #[test]
    fn parent_folder_handles_acronyms() {
        assert_eq!(infer_parent_folder("APIResponseDirs"), "api_response");
        assert_eq!(infer_parent_folder("MJMLTemplates"), "mjml");
    }

    #[test]
    fn parent_folder_falls_back_to_full_name() {
        // Stripping would leave an empty string, so the full name is used.
        assert_eq!(infer_parent_folder("Dirs"), "dirs");
        assert_eq!(infer_parent_folder("Templates"), "templates");
        assert_eq!(infer_parent_folder("Weird"), "weird");
    }

    #[test]
    fn variant_dir_name_applies_case() {
        assert_eq!(
            variant_dir_name("ForgotPassword", Case::Snake),
            "forgot_password"
        );
        assert_eq!(
            variant_dir_name("ForgotPassword", Case::Kebab),
            "forgot-password"
        );
        assert_eq!(
            variant_dir_name("ForgotPassword", Case::Flat),
            "forgotpassword"
        );
        assert_eq!(
            variant_dir_name("TermsOfService", Case::Kebab),
            "terms-of-service"
        );
        assert_eq!(variant_dir_name("APIKey", Case::Snake), "api_key");
    }

    #[test]
    fn parse_case_accepts_known_names() {
        for (name, case) in KNOWN_CASES {
            assert_eq!(parse_case(name), Some(case), "case `{name}`");
        }
    }

    #[test]
    fn parse_case_rejects_unknown_names() {
        assert_eq!(parse_case(""), None);
        assert_eq!(parse_case("Snake"), None);
        assert_eq!(parse_case("camel"), None);
        assert_eq!(parse_case("pascal"), None);
    }
}
