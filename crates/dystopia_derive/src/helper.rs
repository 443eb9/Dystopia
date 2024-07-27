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
