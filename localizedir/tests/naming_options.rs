//! Integration tests for the `case = "kebab"` naming option and
//! `#[dir_name = "..."]` variant overrides.

use localizedir::{LocalizeDir, localizedir};

#[localizedir("tests/fixtures/localized", case = "kebab")]
enum LegalDocs {
    TermsOfService,
    PrivacyPolicy,
}

#[localizedir("tests/fixtures/localized", parent = "email")]
enum RenamedDirs {
    #[dir_name = "newsletter"]
    MailingList,
}

#[test]
fn kebab_case_directory_names() {
    assert_eq!(LegalDocs::TermsOfService.dir_name(), "terms-of-service");
    assert_eq!(LegalDocs::PrivacyPolicy.dir_name(), "privacy-policy");
    assert_eq!(LegalDocs::parent_folder(), "legal");
}

#[test]
fn kebab_directories_resolve_for_every_locale() {
    for locale in LegalDocs::available_locales() {
        let dir = LegalDocs::TermsOfService.to_localized_dir(locale);
        assert_eq!(dir.path(), LegalDocs::TermsOfService.full_path(locale));
    }
}

#[test]
fn dir_name_attribute_overrides_the_variant_name() {
    assert_eq!(RenamedDirs::MailingList.dir_name(), "newsletter");
    let dir = RenamedDirs::MailingList.to_localized_dir("en");
    assert_eq!(dir.path(), RenamedDirs::MailingList.full_path("en"));
}
