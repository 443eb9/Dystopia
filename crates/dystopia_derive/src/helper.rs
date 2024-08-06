#![allow(unused)]

macro_rules! impl_meta_unpack {
    ($fn_name: ident, $target: ident, $non_target_a: ident, $non_target_b: ident, $hint: literal, $returns: ty) => {
        pub fn $fn_name(meta: &syn::Meta) -> &$returns {
            match meta {
                syn::Meta::$target(m) => m,
                syn::Meta::$non_target_a(_) | syn::Meta::$non_target_b(_) => {
                    panic!("This attribute shoud be a {}", $hint)
                }
            }
        }
    };
}

impl_meta_unpack!(
    unpack_name_value,
    NameValue,
    List,
    Path,
    "name value",
    syn::MetaNameValue
);

impl_meta_unpack!(unpack_list, List, NameValue, Path, "list", syn::MetaList);

impl_meta_unpack!(unpack_path, Path, NameValue, List, "path", syn::Path);

macro_rules! impl_data_unpack {
    ($fn_name: ident, $target: ident, $non_target_a: ident, $non_target_b: ident, $hint: literal, $returns: ty) => {
        pub fn $fn_name(data: &syn::Data) -> &$returns {
            match data {
                syn::Data::$target(d) => d,
                syn::Data::$non_target_a(_) | syn::Data::$non_target_b(_) => {
                    panic!("This trait should only be derived for {}s", $hint)
                }
            }
        }
    };
}

impl_data_unpack!(
    unpack_data_struct,
    Struct,
    Enum,
    Union,
    "struct",
    syn::DataStruct
);

impl_data_unpack!(unpack_data_enum, Enum, Struct, Union, "enum", syn::DataEnum);

impl_data_unpack!(
    unpack_data_union,
    Union,
    Struct,
    Enum,
    "union",
    syn::DataUnion
);

pub fn core_crate() -> syn::Ident {
    match proc_macro_crate::crate_name("dystopia_core").unwrap() {
        proc_macro_crate::FoundCrate::Itself => {
            syn::Ident::new("crate", proc_macro2::Span::call_site())
        }
        proc_macro_crate::FoundCrate::Name(name) => {
            syn::Ident::new(&name, proc_macro2::Span::call_site())
        }
    }
}
