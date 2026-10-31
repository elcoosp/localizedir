<div align="center">
  <img src="docs/logo.png" alt="localizedir Logo" width="300"/>
  <p>
    <strong>Compile-Time Localized Directory Inclusion for Rust</strong><br/>
    A powerful procedural macro that automates the inclusion of localized directories, eliminating boilerplate code for internationalization.
  </p>
  <p>
    <img src="https://img.shields.io/badge/Rust-1.94%20%7C%202024-000000?style=flat-square&logo=rust" alt="Rust"/>
    <img src="https://img.shields.io/crates/v/localizedir?style=flat-square" alt="Crates.io"/>
    <img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License MIT"/>
  </p>
</div>

---

`localizedir` streamlines internationalization in Rust by automatically generating localized directory accessors at compile time. By leveraging the `include_dir` crate, it ensures all paths are verified and embedded directly into your binary, providing zero-boilerplate, type-safe access to locale-specific assets.

## Features

- **Zero Boilerplate**: Automatically generates localized directory accessors and helper methods.
- **Compile-Time Safety**: All paths are verified and included at compile time, preventing runtime file-not-found errors.
- **Multi-Locale Support**: Seamlessly handles multiple languages and regions with a simple string argument.
- **Flexible Naming**: Smart parent folder detection derived directly from enum names.
- **IDE Friendly**: Generated code is fully compatible with `rust-analyzer` for excellent autocomplete and navigation.

## Installation

Add `localizedir` and `include_dir` to your `Cargo.toml` dependencies:

```toml
[dependencies]
localizedir = "0.1"
include_dir = "0.7"
```

## Quick Start

Define an enum with the `#[localizedir]` attribute, pointing to your base localized directory:

```rust
use localizedir::localizedir;
use include_dir::include_dir;

#[localizedir("templates/localized")]
enum AuthTemplates {
    Welcome,
    ForgotPassword,
    EmailVerification,
}

fn main() {
    // Access French welcome templates
    let welcome_fr = AuthTemplates::Welcome.to_localized_dir("fr");
    println!("French templates: {}", welcome_fr.path().display());
    
    // Access English forgot password templates
    let forgot_en = AuthTemplates::ForgotPassword.to_localized_dir("en");
    println!("English templates: {}", forgot_en.path().display());
    
    // Get directory metadata
    println!("Parent folder: {}", AuthTemplates::parent_folder());
    println!("Base path: {}", AuthTemplates::base_path());
    println!("Welcome dir name: {}", AuthTemplates::Welcome.dir_name());
}
```

## Directory Structure

The macro expects a specific, predictable directory structure based on the locale, the inferred parent folder, and the kebab-case variant name:

```text
templates/localized/
├── en/
│   ├── auth/
│   │   ├── welcome/
│   │   ├── forgot_password/
│   │   └── email_verification/
├── fr/
│   ├── auth/
│   │   ├── welcome/
│   │   ├── forgot_password/
│   │   └── email_verification/
└── es/
    ├── auth/
        ├── welcome/
        ├── forgot_password/
        └── email_verification/
```

> [!NOTE]
> The parent folder (e.g., `auth`) is automatically inferred by stripping common suffixes like `Templates` or `Documents` from the enum name (`AuthTemplates` -> `auth`) and converting it to snake_case.

## Advanced Usage

### Multiple Directory Sets

You can define multiple enums pointing to the same or different base paths to organize your assets logically:

```rust
#[localizedir("templates/localized")]
enum AuthTemplates {
    Welcome,
    ResetPassword,
}

#[localizedir("templates/localized")]
enum EmailTemplates {
    Newsletter,
    Promotional,
}

#[localizedir("assets/legal")]
enum LegalDocuments {
    TermsOfService,
    PrivacyPolicy,
}
```

### Full Path Information

The macro generates several helper methods for introspection and path resolution:

```rust
let template = AuthTemplates::Welcome;

println!("Directory name: {}", template.dir_name());
println!("Full path for French: {}", template.full_path("fr"));
println!("Parent folder: {}", AuthTemplates::parent_folder());
println!("Base path: {}", AuthTemplates::base_path());
```

## Generated Code

For transparency, applying `#[localizedir("templates/localized")]` to an enum generates code similar to the following:

```rust
enum AuthTemplates {
    Welcome,
    ForgotPassword,
}

static WELCOME_DIR_NAME: &str = "welcome";
static FORGOT_PASSWORD_DIR_NAME: &str = "forgot_password";

impl AuthTemplates {
    pub fn to_localized_dir(&self, locale: &str) -> include_dir::Dir<'static> {
        match self {
            AuthTemplates::Welcome => {
                include_dir::include_dir!(concat!("templates/localized/", locale, "/auth/welcome"))
            }
            AuthTemplates::ForgotPassword => {
                include_dir::include_dir!(concat!("templates/localized/", locale, "/auth/forgot_password"))
            }
        }
    }
    
    // ... plus additional helper methods (dir_name, full_path, parent_folder, base_path)
}
```

## Naming Conventions

The macro automatically converts enum names and variants into directory names using predictable rules:

| Enum Name | Inferred Parent Folder | Variant | Directory Name |
| :--- | :--- | :--- | :--- |
| `AuthTemplates` | `auth` | `Welcome` | `welcome` |
| `EmailTemplates` | `email` | `Newsletter` | `newsletter` |
| `UserProfileTemplates` | `user_profile` | `Settings` | `settings` |

## Common Use Cases

### MJML or HTML Email Templates
```rust
#[localizedir("mjml-dist/localized")]
enum AuthEmails {
    Welcome,
    PasswordReset,
    AccountVerification,
}

fn render_localized_email(template: AuthEmails, locale: &str, data: &EmailData) -> String {
    let dir = template.to_localized_dir(locale);
    // Process MJML templates from the embedded directory
    // ...
}
```

### Static Asset Management
```rust
#[localizedir("assets/localized")]
enum LegalDocuments {
    TermsOfService,
    PrivacyPolicy,
}

fn get_localized_document(doc: LegalDocuments, locale: &str) -> &'static str {
    let dir = doc.to_localized_dir(locale);
    // Extract and return document content from the embedded directory
    // ...
}
```

## Best Practices

> [!TIP]
> - **Organize by Feature**: Group related templates or assets within the same enum to keep your codebase modular.
> - **Consistent Naming**: Use clear, descriptive variant names (e.g., `PasswordReset` instead of `Pass`).
> - **Fallback Systems**: Combine the generated accessors with a runtime fallback mechanism (e.g., trying "fr", then defaulting to "en" if the directory or file is missing).

## Limitations

- Requires the `include_dir` crate as a dependency for directory inclusion.
- Directory paths must exist at compile time; the macro will fail to compile if the base path or inferred subdirectories are missing.
- Locales are specified as string slices (`&str`) at runtime, meaning typoed locale strings will not be caught by the compiler (though the base path structure is validated).

---

> [!IMPORTANT]
> This crate is designed to work in tandem with the [`include_dir`](https://crates.io/crates/include_dir) crate, which embeds directory contents directly into your binary at compile time, ensuring zero runtime I/O overhead for asset retrieval.
