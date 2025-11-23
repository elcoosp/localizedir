# Localizedir

A powerful Rust procedural macro that automates the inclusion of localized directories at compile time, eliminating boilerplate code for internationalization.

## Features

- 🚀 **Zero Boilerplate** - Automatically generates localized directory accessors
- 📁 **Compile-Time Safety** - All paths are verified and included at compile time
- 🌍 **Multi-Locale Support** - Easy support for multiple languages and regions
- 🔧 **Flexible Naming** - Smart parent folder detection from enum names
- 💡 **IDE Friendly** - Generated code works well with Rust analyzers

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
localizedir = "0.1"
include_dir = "0.7"
```

## Quick Start

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
    
    // Get directory information
    println!("Parent folder: {}", AuthTemplates::parent_folder());
    println!("Base path: {}", AuthTemplates::base_path());
    println!("Welcome dir name: {}", AuthTemplates::Welcome.dir_name());
}
```

## Directory Structure

The macro expects this directory structure:

```
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
    │   ├── welcome/
    │   ├── forgot_password/
    │   └── email_verification/
```

## Advanced Usage

### Multiple Directory Sets

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

#[localizedir("templates/localized")]
enum UserProfileTemplates {
    Settings,
    Preferences,
}
```

### Full Path Information

```rust
let template = AuthTemplates::Welcome;
println!("Directory name: {}", template.dir_name());
println!("Full path for French: {}", template.full_path("fr"));
println!("Parent folder: {}", AuthTemplates::parent_folder());
```

## Generated Code

For this enum:
```rust
#[localizedir("templates/localized")]
enum AuthTemplates {
    Welcome,
    ForgotPassword,
}
```

The macro generates approximately:
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
    
    // ... plus other helper methods
}
```

## Naming Conventions

The macro automatically converts enum names to directory names:

| Enum Name | Parent Folder | Directory Name |
|-----------|---------------|----------------|
| `AuthTemplates` | `auth` | `welcome` |
| `EmailTemplates` | `email` | `newsletter` |
| `UserProfileTemplates` | `user_profile` | `settings` |

## Use Cases

### MJML Templates
```rust
#[localizedir("mjml-dist/localized")]
enum AuthEmails {
    Welcome,
    PasswordReset,
    AccountVerification,
}

fn render_localized_email(template: AuthEmails, locale: &str, data: &EmailData) -> String {
    let dir = template.to_localized_dir(locale);
    // Process MJML templates from the directory
    // ...
}
```

### Static Asset Management
```rust
#[localizedir("assets/localized")]
enum LegalDocuments {
    TermsOfService,
    PrivacyPolicy,
    CookiePolicy,
}

fn get_localized_document(doc: LegalDocuments, locale: &str) -> &'static str {
    let dir = doc.to_localized_dir(locale);
    // Extract and return document content
    // ...
}
```

## Configuration

### Custom Base Paths
```rust
#[localizedir("src/templates/emails")]
enum EmailTemplates { /* ... */ }

#[localizedir("../shared-templates")]
enum SharedTemplates { /* ... */ }
```

## Best Practices

1. **Organize by Feature**: Group related templates in the same enum
2. **Consistent Naming**: Use clear, descriptive variant names
3. **Directory Structure**: Maintain the expected locale/parent/variant structure
4. **Error Handling**: Use the provided methods to build robust locale fallback systems

## Limitations

- Requires the `include_dir` crate for directory inclusion
- Directory paths must exist at compile time
- Locales are specified as string literals at runtime

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## License

MIT License - see LICENSE file for details.

---

**Note**: This crate is designed to work with the [`include_dir`](https://crates.io/crates/include_dir) crate, which embeds directory contents in your binary at compile time.
