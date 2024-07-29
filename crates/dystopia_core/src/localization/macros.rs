/// Generate a localizable enum for [`LocalizableStruct`](crate::localization::LocalizableStruct)s.
#[macro_export]
macro_rules! gen_localizable_enum {
    ($name: ident, $($variant: ident),+) => {
        #[derive(LocalizableEnum)]
        pub(super) enum $name {
            $($variant,)*
        }

        impl AsBuiltUiElement for $name {
            type BuiltType = Entity;
        }
    };
}
