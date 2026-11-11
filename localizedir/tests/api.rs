//! Integration tests for the core `#[localizedir]` surface: every trait
//! method, compile-time discovery, and embedded content.

use std::path::PathBuf;

use localizedir::{Dir, LocalizeDir, localizedir};

#[localizedir("tests/fixtures/localized")]
enum AuthDirs {
    Welcome,
    ForgotPassword,
}

#[test]
fn available_locales_are_sorted_and_discovered_at_compile_time() {
    assert_eq!(AuthDirs::available_locales(), &["de", "en", "fr"]);
}

#[test]
fn has_locale_matches_discovered_names_exactly() {
    assert!(AuthDirs::has_locale("en"));
    assert!(!AuthDirs::has_locale("EN"));
    assert!(!AuthDirs::has_locale("zz"));
}

#[test]
fn metadata_reflects_the_attribute() {
    assert_eq!(AuthDirs::base_path(), "tests/fixtures/localized");
    assert_eq!(AuthDirs::parent_folder(), "auth");
    assert_eq!(AuthDirs::fallback_locale(), None);
}

#[test]
fn dir_name_uses_snake_case_by_default() {
    assert_eq!(AuthDirs::Welcome.dir_name(), "welcome");
    assert_eq!(AuthDirs::ForgotPassword.dir_name(), "forgot_password");
}

#[test]
fn full_path_composes_the_conventional_path() {
    // Embedded entry paths are relative to the embedded root.
    assert_eq!(
        AuthDirs::Welcome.full_path("fr"),
        PathBuf::from("fr/auth/welcome")
    );
    // full_path is a pure composition: unknown locales compose too.
    assert_eq!(
        AuthDirs::Welcome.full_path("zz"),
        PathBuf::from("zz/auth/welcome")
    );
}

#[test]
fn try_to_localized_dir_returns_embedded_dirs() {
    for locale in ["de", "en", "fr"] {
        let dir = AuthDirs::Welcome
            .try_to_localized_dir(locale)
            .expect("exists");
        // The returned directory's path equals the composed full_path.
        assert_eq!(dir.path(), AuthDirs::Welcome.full_path(locale));
    }
}

#[test]
fn try_to_localized_dir_returns_none_for_unknown_locale() {
    assert!(AuthDirs::Welcome.try_to_localized_dir("zz").is_none());
}

#[test]
fn embedded_directories_expose_their_files() {
    let dir: &Dir<'static> = AuthDirs::ForgotPassword.to_localized_dir("en");
    // Entry paths inside the embedded tree are relative to the embedded root.
    let file = dir
        .get_file(AuthDirs::ForgotPassword.full_path("en").join("note.txt"))
        .expect("embedded file");
    let contents = file.contents_utf8().map(|s| s.replace("\r\n", "\n"));
    assert_eq!(contents, Some("en:auth:forgot_password\n".to_string()));
}

#[test]
fn to_localized_dir_returns_embedded_dir_for_known_locale() {
    let dir = AuthDirs::Welcome.to_localized_dir("de");
    assert!(dir.path().ends_with("de/auth/welcome"));
}

#[test]
#[should_panic(expected = "no embedded directory for locale `zz`")]
fn to_localized_dir_panics_for_unknown_locale_without_fallback() {
    let _ = AuthDirs::Welcome.to_localized_dir("zz");
}

#[test]
fn each_variant_maps_to_its_own_directory_per_locale() {
    for locale in AuthDirs::available_locales() {
        for variant in [AuthDirs::Welcome, AuthDirs::ForgotPassword] {
            let dir = variant.to_localized_dir(locale);
            assert_eq!(dir.path(), variant.full_path(locale));
            let note = variant.full_path(locale).join("note.txt");
            assert!(dir.get_file(&note).is_some());
        }
    }
}
