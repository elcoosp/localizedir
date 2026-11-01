use localizedir_macros::localizedir;

#[localizedir("whatever", unknow = "en")]
enum UnknownOption {
    Welcome,
}
