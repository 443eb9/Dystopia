/// Provides method `get`.
#[macro_export]
macro_rules! impl_ro_tuple_struct {
    ($ty: ident, $elem: ty) => {
        impl $ty {
            pub fn get(&self) -> &$elem {
                &self.0
            }
        }
    };
}

/// Provides method `get`, `get_mut` and `set`.
#[macro_export]
macro_rules! impl_rw_tuple_struct {
    ($ty: ident, $elem: ty) => {
        impl $ty {
            pub fn get(&self) -> &$elem {
                &self.0
            }

            pub fn get_mut(&mut self) -> &$elem {
                &mut self.0
            }

            pub fn set(&mut self, value: $elem) {
                self.0 = value;
            }
        }
    };
}
