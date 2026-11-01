use localizedir_macros::localizedir;

#[localizedir("whatever")]
enum Generic<T> {
    Welcome(T),
}
