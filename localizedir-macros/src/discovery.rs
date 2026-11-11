//! Compile-time filesystem access: resolving attribute paths (including
//! `include_dir`-style `$VAR` interpolation), discovering locale
//! directories, and validating that every (locale, variant) directory
//! exists.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Directories that must never be mistaken for locales.
const NON_LOCALE_DIRS: [&str; 2] = ["node_modules", "target"];

/// Resolves `$VAR` / `$NAME_WITH_DIGITS` sequences the same way
/// `include_dir!` does, so that validation and embedding always agree on the
/// target directory.
///
/// Mirrors `include_dir_macros::resolve_path`: variables are
/// `[A-Za-z0-9_]+`, looked up once (no recursive expansion).
pub(crate) fn resolve_path(raw: &str) -> Result<String, String> {
    resolve_path_with(raw, |variable| std::env::var(variable).ok())
}

/// [`resolve_path`] with an injectable variable lookup, mirroring
/// `include_dir_macros::resolve_path` (which also takes a `get_env`
/// closure); injectable so tests never touch process environment.
pub(crate) fn resolve_path_with(
    raw: &str,
    get_env: impl Fn(&str) -> Option<String>,
) -> Result<String, String> {
    let mut resolved = String::with_capacity(raw.len());
    let mut unprocessed = raw;

    while let Some(dollar) = unprocessed.find('$') {
        let (head, tail) = unprocessed.split_at(dollar);
        resolved.push_str(head);

        let variable: String = tail
            .chars()
            .skip(1)
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if variable.is_empty() {
            return Err(format!(
                "invalid `$` variable syntax in `{raw}`; expected `$NAME` where NAME is alphanumeric or `_`"
            ));
        }

        let Some(value) = get_env(&variable) else {
            return Err(format!(
                "environment variable `${variable}` (used in `{raw}`) is not set"
            ));
        };
        resolved.push_str(&value);
        let consumed = 1 + variable.chars().count();
        unprocessed = &tail[consumed..];
    }

    resolved.push_str(unprocessed);
    Ok(resolved)
}

/// A base path prepared for both validation and embedding.
#[derive(Debug)]
pub(crate) struct AnchoredPath {
    /// Path passed to `include_dir!` (a single string literal); uses the
    /// `$CARGO_MANIFEST_DIR` convention for relative inputs so the embedded
    /// root never depends on the compiler's working directory.
    pub(crate) include_arg: String,
    /// Fully expanded path (absolute) used for validation and runtime lookups.
    pub(crate) expanded: String,
}

/// Anchors the raw attribute path against the package manifest directory.
///
/// - `$VAR`s are expanded exactly like `include_dir!` does.
/// - Absolute paths (including `$CARGO_MANIFEST_DIR/...` inputs) are used as-is.
/// - Relative paths are joined onto `CARGO_MANIFEST_DIR`, because the
///   compiler's working directory is the *workspace* root, not the package
///   root, and `include_dir!` resolves paths against that working directory.
pub(crate) fn anchor_path(raw: &str, manifest_dir: Option<&str>) -> Result<AnchoredPath, String> {
    let resolved = resolve_path(raw)?;
    if Path::new(&resolved).is_absolute() {
        return Ok(AnchoredPath {
            include_arg: raw.to_owned(),
            expanded: resolved,
        });
    }

    let Some(manifest) = manifest_dir else {
        return Err(
            "relative base paths require `CARGO_MANIFEST_DIR` to be set; build through cargo"
                .to_owned(),
        );
    };
    let expanded = Path::new(manifest)
        .join(&resolved)
        .to_string_lossy()
        .into_owned();
    Ok(AnchoredPath {
        include_arg: format!("$CARGO_MANIFEST_DIR/{resolved}"),
        expanded,
    })
}

/// Discovers locale directories under `base_path`.
///
/// Returns the sorted, de-duplicated names of all immediate subdirectories
/// that pass [`is_locale_dir_name`], or a user-facing error message.
pub(crate) fn discover_locales(base_path: &Path) -> Result<Vec<String>, String> {
    let display = base_path.display();

    if !base_path.exists() {
        return Err(format!(
            "base path does not exist: {display} (paths are resolved relative to the crate manifest directory)"
        ));
    }
    if !base_path.is_dir() {
        return Err(format!("base path is not a directory: {display}"));
    }

    let mut locales = BTreeSet::new();
    for entry in fs::read_dir(base_path)
        .map_err(|error| format!("cannot read base path {display}: {error}"))?
    {
        let entry = entry.map_err(|error| format!("cannot read entry in {display}: {error}"))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(std::ffi::OsStr::to_str) else {
            continue;
        };
        if is_locale_dir_name(name) {
            locales.insert(name.to_owned());
        }
    }

    if locales.is_empty() {
        return Err(format!(
            "no locale directories found in {display}; expected at least one subdirectory (e.g. `en/`)"
        ));
    }

    Ok(locales.into_iter().collect())
}

/// `true` for directory names that should be treated as locales.
fn is_locale_dir_name(name: &str) -> bool {
    !name.starts_with('.') && !name.starts_with('_') && !NON_LOCALE_DIRS.contains(&name)
}

/// Validates that every variant directory exists for every locale.
///
/// Returns the list of missing `(locale, dir_name)` relative paths.
pub(crate) fn missing_variant_dirs(
    base_path: &Path,
    locales: &[String],
    parent_folder: &str,
    dir_names: &[String],
) -> Vec<PathBuf> {
    let mut missing = Vec::new();
    for locale in locales {
        for dir_name in dir_names {
            let candidate = base_path.join(locale).join(parent_folder).join(dir_name);
            if !candidate.is_dir() {
                missing.push(candidate);
            }
        }
    }
    missing
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::{
        anchor_path, discover_locales, is_locale_dir_name, missing_variant_dirs, resolve_path,
        resolve_path_with,
    };

    fn write_tree(root: &Path, dirs: &[&str]) {
        for dir in dirs {
            fs::create_dir_all(root.join(dir)).expect("create fixture dir");
        }
    }

    #[test]
    fn resolve_path_without_variables_is_identity() {
        assert_eq!(
            resolve_path("templates/localized").as_deref(),
            Ok("templates/localized")
        );
        assert_eq!(resolve_path("").as_deref(), Ok(""));
    }

    #[test]
    fn resolve_path_interpolates_variables() {
        // Mirrors include_dir's own unit test: `$TOP_LEVEL.txt` stops at `.`.
        let get_env = |name: &str| (name == "LOCALIZEDIR_TEST_VAR").then(|| "resolved".to_owned());
        assert_eq!(
            resolve_path_with("$LOCALIZEDIR_TEST_VAR/locales", get_env).as_deref(),
            Ok("resolved/locales")
        );
        assert_eq!(
            resolve_path_with("a/$LOCALIZEDIR_TEST_VAR.txt/b", get_env).as_deref(),
            Ok("a/resolved.txt/b")
        );
        assert_eq!(
            resolve_path_with("$LOCALIZEDIR_TEST_VAR$LOCALIZEDIR_TEST_VAR", get_env).as_deref(),
            Ok("resolvedresolved")
        );
    }

    #[test]
    fn resolve_path_hits_the_real_environment() {
        // `PATH` is set in every environment cargo tests run in; this covers
        // the `std::env::var` wrapper without mutating process state.
        assert!(resolve_path("$PATH").is_ok_and(|resolved| !resolved.contains('$')));
    }

    #[test]
    fn resolve_path_rejects_bad_syntax_and_missing_vars() {
        let error = resolve_path_with("a/$/b", |_| None).unwrap_err();
        assert!(error.contains("invalid `$` variable syntax"), "{error}");

        let error = resolve_path_with("$LOCALIZEDIR_DEFINITELY_UNSET_VAR/x", |_| None).unwrap_err();
        assert!(
            error.contains("`$LOCALIZEDIR_DEFINITELY_UNSET_VAR`") && error.contains("is not set"),
            "{error}"
        );
    }

    #[test]
    fn anchor_path_passes_absolute_inputs_through() {
        let anchored = anchor_path("/abs/locales", Some("/pkg")).expect("anchors");
        assert_eq!(anchored.include_arg, "/abs/locales");
        assert_eq!(anchored.expanded, "/abs/locales");

        // `$CARGO_MANIFEST_DIR` is expanded by `resolve_path` against the
        // real environment (cargo always sets it), yielding an absolute path.
        let anchored = anchor_path("$CARGO_MANIFEST_DIR/locales", Some("/pkg")).expect("anchors");
        assert_eq!(anchored.include_arg, "$CARGO_MANIFEST_DIR/locales");
        let expected = format!(
            "{}/locales",
            std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR for tests")
        );
        assert_eq!(anchored.expanded, expected);
    }

    #[test]
    fn anchor_path_joins_relative_paths_onto_the_manifest_dir() {
        let anchored = anchor_path("templates/localized", Some("/pkg")).expect("anchors");
        assert_eq!(
            anchored.include_arg,
            "$CARGO_MANIFEST_DIR/templates/localized"
        );
        assert_eq!(anchored.expanded, "/pkg/templates/localized");

        let error = anchor_path("$DATA/locales", Some("/pkg")).unwrap_err();
        assert!(
            error.contains("`$DATA`") && error.contains("is not set"),
            "{error}"
        );

        let error = anchor_path("templates/localized", None).unwrap_err();
        assert!(error.contains("CARGO_MANIFEST_DIR"), "{error}");
    }

    #[test]
    fn discover_locales_reads_sorted_subdirectories() {
        let temp = TempDir::new().expect("tempdir");
        write_tree(
            temp.path(),
            &[
                "fr",
                "en",
                "de",
                ".hidden",
                "_partials",
                "node_modules",
                "target",
            ],
        );
        let locales = discover_locales(temp.path()).expect("locales");
        assert_eq!(locales, vec!["de", "en", "fr"]);
    }

    #[test]
    fn discover_locales_ignores_files() {
        let temp = TempDir::new().expect("tempdir");
        fs::write(temp.path().join("README.md"), "not a locale").expect("write file");
        assert!(discover_locales(temp.path()).is_err());
    }

    #[cfg(unix)]
    #[test]
    #[cfg(not(target_os = "macos"))]
    fn discover_locales_skips_non_utf8_directory_names() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let temp = TempDir::new().expect("tempdir");
        let invalid = OsStr::from_bytes(b"\xff\xfe-not-utf8");
        fs::create_dir_all(temp.path().join(invalid)).expect("create non-utf8 dir");
        fs::create_dir_all(temp.path().join("en")).expect("create en");

        let locales = discover_locales(temp.path()).expect("locales");
        assert_eq!(locales, vec!["en"]);
    }

    #[test]
    fn discover_locales_reports_missing_path() {
        let temp = TempDir::new().expect("tempdir");
        let error = discover_locales(&temp.path().join("missing")).unwrap_err();
        assert!(error.contains("base path does not exist"), "{error}");
    }

    #[test]
    fn discover_locales_reports_non_directory() {
        let temp = TempDir::new().expect("tempdir");
        let file = temp.path().join("a-file");
        fs::write(&file, "x").expect("write file");
        let error = discover_locales(&file).unwrap_err();
        assert!(error.contains("is not a directory"), "{error}");
    }

    #[test]
    fn discover_locales_reports_empty_directory() {
        let temp = TempDir::new().expect("tempdir");
        let error = discover_locales(temp.path()).unwrap_err();
        assert!(error.contains("no locale directories found"), "{error}");
    }

    #[test]
    fn locale_dir_name_filters() {
        assert!(is_locale_dir_name("en"));
        assert!(is_locale_dir_name("pt-BR"));
        assert!(!is_locale_dir_name(".git"));
        assert!(!is_locale_dir_name("_partials"));
        assert!(!is_locale_dir_name("node_modules"));
        assert!(!is_locale_dir_name("target"));
    }

    #[test]
    fn missing_variant_dirs_lists_every_gap() {
        let temp = TempDir::new().expect("tempdir");
        write_tree(
            temp.path(),
            &[
                "en/auth/welcome",
                "en/auth/forgot_password",
                "fr/auth/welcome",
            ],
        );
        let locales = vec!["en".to_owned(), "fr".to_owned()];
        let missing = missing_variant_dirs(
            temp.path(),
            &locales,
            "auth",
            &["welcome".to_owned(), "forgot_password".to_owned()],
        );
        assert_eq!(missing.len(), 1);
        assert_eq!(
            missing[0],
            temp.path().join("fr/auth/forgot_password"),
            "only the fr/forgot_password directory is missing"
        );
    }
}
