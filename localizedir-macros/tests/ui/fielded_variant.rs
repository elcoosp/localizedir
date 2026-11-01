use localizedir_macros::localizedir;

#[localizedir("whatever")]
enum Fielded {
    Welcome { language: &'static str },
}
