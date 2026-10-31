pub use localizedir_macros::localizedir;
pub trait LocalizeDir {
    fn to_localized_dir(&self, locale: &str) -> &'static include_dir::Dir<'static>;
    fn available_locales() -> &'static [&'static str];
    fn parent_folder() -> &'static str;
    fn dir_name(&self) -> &'static str;
}
