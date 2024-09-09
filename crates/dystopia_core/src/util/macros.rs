/// A utility to generate `new` method for single element tuple structs.
#[macro_export]
macro_rules! tuple_struct_new {
    ($ty: ty, $val: ty) => {
        impl $ty {
            pub fn new(value: $val) -> Self {
                Self(value)
            }
        }
    };
}
