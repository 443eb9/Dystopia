use crate::localization::{LangFile, LocalizableData};

macro_rules! impl_localizable_number {
    ($ty:ty, $nan:expr) => {
        impl LocalizableData for $ty {
            fn localize(&self, _lang: &LangFile) -> String {
                if *self == $nan {
                    "NaN".into()
                } else {
                    self.to_string()
                }
            }
        }
    };
}

impl_localizable_number!(i8, i8::MAX);
impl_localizable_number!(i16, i16::MAX);
impl_localizable_number!(i32, i32::MAX);
impl_localizable_number!(i64, i64::MAX);
impl_localizable_number!(i128, i128::MAX);

impl_localizable_number!(u8, u8::MAX);
impl_localizable_number!(u16, u16::MAX);
impl_localizable_number!(u32, u32::MAX);
impl_localizable_number!(u64, u64::MAX);
impl_localizable_number!(u128, u128::MAX);

impl LocalizableData for f32 {
    fn localize(&self, _lang: &LangFile) -> String {
        format!("{:.2}", self)
    }
}

impl LocalizableData for f64 {
    fn localize(&self, _lang: &LangFile) -> String {
        format!("{:.2}", self)
    }
}
