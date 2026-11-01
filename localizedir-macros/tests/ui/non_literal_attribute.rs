use localizedir_macros::localizedir;

#[localizedir(not_a_string_literal)]
enum WrongAttr {
    Welcome,
}
