//! Integration tests for the `fallback = "..."` and `parent = "..."`
//! attribute options.

use localizedir::{LocalizeDir, localizedir};

#[localizedir("tests/fixtures/localized", parent = "email", fallback = "en")]
enum EmailFallback {
    Newsletter,
    Promotional,
}

#[test]
fn parent_override_replaces_inference() {
    // Without the override the enum would infer `email_fallback`.
    assert_eq!(EmailFallback::parent_folder(), "email");
}

#[test]
fn fallback_locale_is_exposed() {
    assert_eq!(EmailFallback::fallback_locale(), Some("en"));
}

#[test]
fn try_lookup_stays_pure_even_with_fallback_configured() {
    // `xx` was never discovered, so the Option API reports None...
    assert!(
        EmailFallback::Newsletter
            .try_to_localized_dir("xx")
            .is_none()
    );
    // ...and known locales still resolve directly.
    let dir = EmailFallback::Newsletter
        .try_to_localized_dir("fr")
        .expect("fr");
    assert_eq!(dir.path(), EmailFallback::Newsletter.full_path("fr"));
}

#[test]
fn panicking_lookup_falls_back_to_the_configured_locale() {
    let dir = EmailFallback::Promotional.to_localized_dir("xx");
    assert_eq!(dir.path(), EmailFallback::Promotional.full_path("en"));
    assert_eq!(
        dir.get_file(EmailFallback::Promotional.full_path("en").join("note.txt"))
            .expect("file")
            .contents_utf8(),
        Some("en:email:promotional\n")
    );
}

#[test]
fn fallback_never_shadows_known_locales() {
    let dir = EmailFallback::Newsletter.to_localized_dir("de");
    assert_eq!(dir.path(), EmailFallback::Newsletter.full_path("de"));
}

#[localizedir("tests/fixtures/localized", parent = "email", fallback = "xx")]
enum BrokenFallback {
    Newsletter,
}

#[test]
#[should_panic(
    expected = "no embedded directory for variant `newsletter` with locale `zz` nor fallback `xx`"
)]
fn missing_request_and_fallback_panics_with_both_names() {
    // `xx` is not a discovered locale, so neither the request nor the
    // fallback can resolve; the panic must name both.
    let _ = BrokenFallback::Newsletter.to_localized_dir("zz");
}
