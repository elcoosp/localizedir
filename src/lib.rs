// Cargo.toml dependencies:
// [dependencies]
// include_dir = "0.7"
//
// [lib]
// proc-macro = true
//
// [dependencies]
// syn = { version = "2.0", features = ["full"] }
// quote = "1.0"
// proc-macro2 = "1.0"

use proc_macro::TokenStream;
use quote::quote;
use std::fs;
use std::path::Path;
use syn::{Data, DeriveInput, Fields, Lit, parse_macro_input};

// /// Generates a method to get localized directories based on locale and variant name
// ///
// /// # Example
// /// ```rust
// /// use localizedir::localizedir;
// /// use include_dir::include_dir;
// /// #[localizedir("mjml-dist/localized")]
// /// enum AuthDirs {
// ///     Welcome,
// ///     Forgot,
// /// }
// ///
// /// // Generates:
// /// // impl AuthDirs {
// /// //     fn to_localized_dir(&self, locale: &str) -> &'static include_dir::Dir<'static> {
// /// //         match (self, locale) {
// /// //             (AuthDirs::Welcome, "fr") => {
// /// //                 static WELCOME_FR: include_dir::Dir<'static> =
// /// //                     include_dir!("mjml-dist/localized/fr/auth/welcome");
// /// //                 &WELCOME_FR
// /// //             }
// /// //             ...
// /// //         }
// /// //     }
// /// // }
// /// ```
#[proc_macro_attribute]
pub fn localizedir(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let base_path = parse_macro_input!(attr as Lit);

    let base_path_str = match base_path {
        Lit::Str(s) => s.value(),
        _ => panic!("localizedir attribute must be a string literal"),
    };

    let enum_name = &input.ident;

    // Extract the parent folder from the enum name
    // e.g., "AuthDirs" -> "auth", "EmailDirs" -> "email"
    let parent_folder = extract_parent_folder(enum_name.to_string());

    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("localizedir can only be used on enums"),
    };

    // Check for unit variants only
    for variant in variants {
        match &variant.fields {
            Fields::Unit => {}
            _ => panic!("localizedir only supports unit variants (no fields)"),
        }
    }

    // Dynamically discover available locales from the filesystem
    let locales = discover_locales(&base_path_str).expect(&format!(
        "Failed to discover locales in path: {}",
        base_path_str
    ));

    // Generate match arms for each variant and locale combination
    let match_arms: Vec<_> = variants.iter().flat_map(|variant| {
        let variant_name = &variant.ident;
        let dir_name = variant_name.to_string().to_lowercase();

        locales.iter().map(|locale| {
            let static_name = syn::Ident::new(
                &format!("{}_{}", dir_name.to_uppercase(), locale.to_uppercase()),
                variant_name.span()
            );

            let full_path = format!(
                "{}/{}/{}/{}",
                base_path_str, locale, parent_folder, dir_name
            );

            // Verify the path exists at compile time
            if !Path::new(&full_path).exists() {
                panic!(
                    "Directory not found: {}. Make sure the directory structure matches the expected pattern.",
                    full_path
                );
            }

            quote! {
                (#enum_name::#variant_name, #locale) => {
                    static #static_name: include_dir::Dir<'static> =
                        include_dir::include_dir!(#full_path);
                    &#static_name
                }
            }
        }).collect::<Vec<_>>()
    }).collect();

    // Collect variant names and identifiers for the dir_name match
    let variant_names: Vec<_> = variants.iter().map(|v| &v.ident).collect();
    let variant_dir_names: Vec<_> = variants
        .iter()
        .map(|v| v.ident.to_string().to_lowercase())
        .collect();

    let output = quote! {
        #input

        impl #enum_name {
            /// Returns a reference to the localized directory for the given locale
            ///
            /// # Arguments
            /// * `locale` - The locale string (e.g., "en", "fr", "es")
            ///
            /// # Panics
            /// Panics if the locale is not supported
            pub fn to_localized_dir(&self, locale: &str) -> &'static include_dir::Dir<'static> {
                match (self, locale) {
                    #(#match_arms)*
                    _ => panic!("Unsupported locale: {}", locale),
                }
            }

            /// Returns all available locales for this directory set
            pub fn available_locales() -> &'static [&'static str] {
                &[#(#locales),*]
            }

            /// Returns the parent folder name for this directory set
            pub fn parent_folder() -> &'static str {
                #parent_folder
            }

            /// Returns the directory name for this variant (lowercase)
            pub fn dir_name(&self) -> &'static str {
                match self {
                    #(#enum_name::#variant_names => #variant_dir_names,)*
                }
            }
        }
    };

    TokenStream::from(output)
}

/// Discovers available locales by scanning the base directory
/// Returns a sorted list of locale directory names
fn discover_locales(base_path: &str) -> Result<Vec<String>, std::io::Error> {
    let path = Path::new(base_path);

    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Base path does not exist: {}", base_path),
        ));
    }

    if !path.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Base path is not a directory: {}", base_path),
        ));
    }

    let mut locales = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        // Only include directories
        if entry_path.is_dir() {
            if let Some(locale_name) = entry_path.file_name() {
                if let Some(locale_str) = locale_name.to_str() {
                    // Filter out hidden directories and common non-locale folders
                    if !locale_str.starts_with('.')
                        && !locale_str.starts_with('_')
                        && locale_str != "node_modules"
                        && locale_str != "target"
                    {
                        locales.push(locale_str.to_string());
                    }
                }
            }
        }
    }

    if locales.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No locale directories found in: {}", base_path),
        ));
    }

    // Sort for consistent output
    locales.sort();

    Ok(locales)
}

/// Extracts the parent folder name from the enum name
/// e.g., "AuthDirs" -> "auth", "EmailDirs" -> "email", "UserProfileDirs" -> "user_profile"
fn extract_parent_folder(enum_name: String) -> String {
    // Remove common suffixes
    let name = enum_name
        .strip_suffix("Dirs")
        .or_else(|| enum_name.strip_suffix("Dir"))
        .unwrap_or(&enum_name);

    // Convert PascalCase to snake_case
    let mut result = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_parent_folder() {
        assert_eq!(extract_parent_folder("AuthDirs".to_string()), "auth");
        assert_eq!(extract_parent_folder("EmailDirs".to_string()), "email");
        assert_eq!(
            extract_parent_folder("UserProfileDirs".to_string()),
            "user_profile"
        );
        assert_eq!(
            extract_parent_folder("APIResponseDirs".to_string()),
            "a_p_i_response"
        );
    }

    #[test]
    fn test_discover_locales() {
        // This test would need a test directory structure to work
        // In real usage, the macro discovers locales at compile time
        use std::fs;
        use std::path::Path;

        // Create a temporary test directory structure
        let test_dir = "test_locales_temp";
        if Path::new(test_dir).exists() {
            fs::remove_dir_all(test_dir).ok();
        }

        fs::create_dir_all(format!("{}/en", test_dir)).unwrap();
        fs::create_dir_all(format!("{}/fr", test_dir)).unwrap();
        fs::create_dir_all(format!("{}/es", test_dir)).unwrap();
        fs::create_dir_all(format!("{}/.hidden", test_dir)).unwrap();

        let locales = discover_locales(test_dir).unwrap();
        assert_eq!(locales, vec!["en", "es", "fr"]);

        // Cleanup
        fs::remove_dir_all(test_dir).ok();
    }
}

// Example usage in another crate:
//
// use include_dir::Dir;
// use your_macro_crate::localizedir;
//
// #[localizedir("mjml-dist/localized")]
// enum AuthDirs {
//     Welcome,
//     Forgot,
// }
//
// fn main() {
//     let welcome_dir = AuthDirs::Welcome.to_localized_dir("fr");
//     let forgot_dir = AuthDirs::Forgot.to_localized_dir("en");
//
//     println!("Available locales: {:?}", AuthDirs::available_locales());
//     println!("Parent folder: {}", AuthDirs::parent_folder());
// }
