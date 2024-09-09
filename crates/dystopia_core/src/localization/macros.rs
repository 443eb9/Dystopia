/// Generate a localizable enum for [`LocalizableStruct`](crate::localization::LocalizableStruct)s.
#[macro_export]
macro_rules! localizable_enum {
    ($name: ident, $access: vis, $($variant: ident),+) => {
        #[derive(dystopia_derive::LocalizableEnum, Clone)]
        $access enum $name {
            $($variant,)*
        }
    };

    ($name: ident, $($variant: ident),+) => {
        #[derive(dystopia_derive::LocalizableEnum, Clone)]
        enum $name {
            $($variant,)*
        }
    };
}
