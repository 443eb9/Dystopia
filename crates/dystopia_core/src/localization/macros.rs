/// Generate a localizable enum for [`LocalizableStruct`](crate::localization::LocalizableStruct)s.
#[macro_export]
macro_rules! gen_localizable_enum {
    ($name: ident, $access: vis, $($variant: ident),+) => {
        #[derive(LocalizableEnum)]
        $access enum $name {
            $($variant,)*
        }
    };

    ($name: ident, $($variant: ident),+) => {
        #[derive(LocalizableEnum)]
        enum $name {
            $($variant,)*
        }
    };
}
