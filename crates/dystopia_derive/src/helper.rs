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

impl_meta_unpack!(
    unpack_list,
    List,
    NameValue,
    Path,
    "list",
    syn::MetaList
);

impl_meta_unpack!(
    unpack_path,
    Path,
    NameValue,
    List,
    "path",
    syn::Path
);
