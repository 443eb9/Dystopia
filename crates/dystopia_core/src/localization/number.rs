use crate::localization::{LangFile, LocalizablePrimitive};

macro_rules! impl_localizable_integer {
    ($ty:ty, $nan:expr) => {
        impl LocalizablePrimitive for $ty {
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

macro_rules! impl_localizable_float {
    ($ty:ty) => {
        impl LocalizablePrimitive for $ty {
            fn localize(&self, _lang: &LangFile) -> String {
                if self.is_nan() {
                    "NaN".into()
                } else {
                    format!("{:.2}", self)
                }
            }
        }
    };
}

impl_localizable_integer!(i8, i8::MAX);
impl_localizable_integer!(i16, i16::MAX);
impl_localizable_integer!(i32, i32::MAX);
impl_localizable_integer!(i64, i64::MAX);
impl_localizable_integer!(i128, i128::MAX);

impl_localizable_integer!(u8, u8::MAX);
impl_localizable_integer!(u16, u16::MAX);
impl_localizable_integer!(u32, u32::MAX);
impl_localizable_integer!(u64, u64::MAX);
impl_localizable_integer!(u128, u128::MAX);

impl_localizable_float!(f32);
impl_localizable_float!(f64);
