<div align="center">
  <img src="https://cdn.jsdelivr.net/gh/elcoosp/localizedir@main/docs/logo.png" alt="localizedir Logo" width="300"/>
  <p>
    <strong>Compile-Time Localized Directory Inclusion for Rust</strong><br/>
    A procedural macro that embeds your localized directories once and generates
    locale-aware accessors — no boilerplate, no runtime I/O.
  </p>
  <p>
    <img src="https://img.shields.io/badge/rustc-1.88%2B-000000?style=flat-square&logo=rust" alt="MSRV"/>
    <img src="https://img.shields.io/crates/v/localizedir?style=flat-square" alt="Crates.io"/>
    <img src="https://img.shields.io/docsrs/localizedir?style=flat-square" alt="docs.rs"/>
    <img src="https://img.shields.io/github/actions/workflow/status/elcoosp/localizedir/ci.yml?branch=main&style=flat-square&label=CI" alt="CI"/>
    <img src="https://img.shields.io/endpoint?style=flat-square&url=https%3A%2F%2Fraw.githubusercontent.com%2Felcoosp%2Flocalizedir%2Fmain%2Fdocs%2Fcoverage.json" alt="Coverage 100%"/>
    <img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License MIT"/>
  </p>
</div>

---

`localizedir` turns an enum into locale-aware accessors over directories
embedded with [`include_dir`](https://crates.io/crates/include_dir). The whole
base directory is embedded **once** at compile time; every lookup is a cheap
navigation inside that embedded tree. Locales are discovered from the
filesystem at compile time, so typos in the *directory structure* fail the
build — and unknown locales at runtime return `Option` instead of file-not-found.

## Features

- **Zero boilerplate**: one attribute generates the full `LocalizeDir`
  implementation, including metadata accessors.
- **Compile-time safety**: missing base paths, missing `(locale, variant)`
  directories, duplicate directory names, and malformed attributes all fail
  to compile with an error pointing at the offending token.
- **One embed, many lookups**: the base directory is embedded a single time
  (`include_dir!` per (variant × locale) would re-expand the tree over and
  over); lookups navigate the embedded tree, so compiles stay fast.
- **Non-panicking core**: `try_to_localized_dir` returns `Option`; the
  panicking `to_localized_dir` layers an optional `fallback` locale on top.
- **Workspace-proof paths**: relative base paths are anchored at
  `$CARGO_MANIFEST_DIR` (the crate manifest directory), so the macro works in
  cargo workspaces where the compiler's working directory is the workspace
  root. `$VAR` interpolation matches `include_dir!`'s own semantics.
- **IDE friendly**: generated code is plain, documented, and fully navigable
  with rust-analyzer.

## Installation

Add `localizedir` and `include_dir` to your `Cargo.toml`:

```toml
[dependencies]
localizedir = "0.2"
include_dir = "0.7"
```

> `include_dir` is a direct dependency of your crate because the macro
> expands to `include_dir::include_dir!(...)`, which resolves in *your* crate.

## Quick Start

```rust
use localizedir::{LocalizeDir, localizedir};

#[localizedir("tests/fixtures/localized")]
enum AuthDirs {
    Welcome,
    ForgotPassword,
}

// Non-panicking lookup returns the embedded directory for a locale.
let dir = AuthDirs::Welcome.try_to_localized_dir("fr").expect("fr exists");
assert!(dir.get_file("fr/auth/welcome/note.txt").is_some());

// Panicking lookup, plus compile-time metadata:
let dir = AuthDirs::Welcome.to_localized_dir("en");
assert_eq!(AuthDirs::available_locales(), &["de", "en", "fr"]);
assert_eq!(AuthDirs::parent_folder(), "auth");
assert_eq!(AuthDirs::base_path(), "tests/fixtures/localized");
assert_eq!(AuthDirs::Welcome.dir_name(), "welcome");
assert!(AuthDirs::has_locale("de"));
assert_eq!(
    AuthDirs::ForgotPassword.full_path("fr"),
    std::path::PathBuf::from("fr/auth/forgot_password"),
);
```

## Directory Structure

The macro expects one subdirectory per locale under the base path, with the
parent folder and per-variant directories inside:

```text
templates/localized/
├── de/
│   └── auth/
│       ├── welcome/
│       └── forgot_password/
├── en/
│   └── auth/
│       ├── welcome/
│       └── forgot_password/
└── fr/
    └── auth/
        ├── welcome/
        └── forgot_password/
```

> [!NOTE]
> The parent folder (e.g. `auth`) is inferred from the enum name by stripping
> a known suffix (`Dirs`, `Dir`, `Templates`, `Template`, `Documents`,
> `Document`, `Directories`, `Docs`, `Assets`, `Asset`, `Files`, `File`,
> `Locales`, `Locale`) and converting the remainder to `snake_case`
> (`AuthTemplates` → `auth`, `APIResponseDirs` → `api_response`).

## Attribute Options

| Option | Default | Meaning |
|--------|---------|---------|
| `"<base path>"` | — | Directory containing one subdirectory per locale. Resolved relative to the crate manifest directory; `$VAR` interpolation supported. |
| `fallback = "<locale>"` | none | Locale used by `to_localized_dir` when the requested locale has no embedded directory. |
| `parent = "<folder>"` | inferred | Overrides the parent folder inferred from the enum name. |
| `case = "snake" \| "kebab" \| "flat"` | `"snake"` | Naming scheme used to convert variant names into directory names. |

Variants may carry `#[dir_name = "..."]` to override an individual directory
name:

```rust
#[localizedir("assets/localized")]
enum MailTemplates {
    #[dir_name = "welcome_v2"]
    Welcome,
    PasswordReset,
}
```

## API

Every annotated enum implements [`LocalizeDir`]:

| Method | Panics? | Description |
|--------|---------|-------------|
| `try_to_localized_dir(&self, locale)` | no | `Option<&'static Dir<'static>>` — the core lookup. |
| `to_localized_dir(&self, locale)` | yes | Panicking lookup; honors `fallback` when configured. |
| `available_locales()` | no | Sorted locales discovered at compile time. |
| `has_locale(locale)` | no | `true` when the locale was discovered. |
| `fallback_locale()` | no | The configured fallback, if any. |
| `base_path()` | no | The base path exactly as written in the attribute. |
| `parent_folder()` | no | Inferred or overridden parent folder. |
| `dir_name(&self)` | no | This variant's directory name. |
| `full_path(&self, locale)` | no | Embedded path (root-relative); equals `Dir::path()` of the returned directory. |

> [!TIP]
> Entry paths inside the embedded tree are relative to the embedded root —
> look files up with `dir.get_file(template.full_path(locale).join("file.txt"))`
> or iterate with `dir.files()`.

## Common Use Cases

### Localized email templates

```rust
#[localizedir("mjml-dist/localized", fallback = "en")]
enum AuthEmails {
    Welcome,
    PasswordReset,
    AccountVerification,
}

fn render_email(template: AuthEmails, locale: &str) -> String {
    let dir = template.to_localized_dir(locale); // falls back to `en` if needed
    let body = dir.get_file(template.full_path(locale).join("index.mjml")).unwrap();
    // hand `body.contents()` to your MJML renderer...
}
```

### Legal documents with kebab-case directories

```rust
#[localizedir("assets/legal", case = "kebab")]
enum LegalDocs {
    TermsOfService,
    PrivacyPolicy,
}

// LegalDocs::TermsOfService.dir_name() == "terms-of-service"
```

## Naming Conventions

| Enum Name | Inferred Parent | Variant | Directory Name (snake) | Directory Name (kebab) |
| :--- | :--- | :--- | :--- | :--- |
| `AuthDirs` | `auth` | `Welcome` | `welcome` | `welcome` |
| `EmailTemplates` | `email` | `Newsletter` | `newsletter` | `newsletter` |
| `UserProfileDirs` | `user_profile` | `ForgotPassword` | `forgot_password` | `forgot-password` |
| `APIResponseDirs` | `api_response` | `HtmlReport` | `html_report` | `html-report` |

## Error Diagnostics

The macro reports every problem at compile time, at the right location:

```text
error: missing directories for some (locale, variant) pairs:
  - /path/to/crate/locales/fr/auth/forgot_password
 --> src/lib.rs:3:15
  |
3 | #[localizedir("locales")]
  |               ^^^^^^^^^
```

Other diagnostics cover non-enum targets, generic enums, empty enums,
fielded variants, duplicate directory names, unknown or duplicate attribute
options, non-literal values, and unset `$VAR`s.

## MSRV & Compatibility

- Minimum Supported Rust Version: **1.88** (edition 2024; trybuild requires it), enforced by CI.
- Tested on Linux, macOS, and Windows.
- Dependencies are refreshed and audited automatically (Dependabot +
  `cargo-deny`).

## Development

```bash
cargo test --workspace --all-features   # unit + integration + trybuild + doctests
cargo clippy --workspace --all-targets --all-features -- -D warnings
./scripts/check-coverage.sh             # enforces 100% line coverage
```

The changelog is generated with [git-cliff](https://git-cliff.org) from
conventional commits; releases are automated in
[`.github/workflows/release.yml`](https://github.com/elcoosp/localizedir/blob/main/github/workflows/release.yml).

