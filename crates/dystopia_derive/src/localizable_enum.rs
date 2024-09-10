pub fn expand_localizable_enum(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = &input.ident;

    let data = crate::helper::unpack_data_enum(&input.data);
    let core_crate = crate::helper::core_crate();
    let mut variants = Vec::with_capacity(data.variants.len());

    for variant in &data.variants {
        let var_ident = &variant.ident;
        if variant.fields.is_empty() {
            variants.push(quote::quote! {
                Self::#var_ident => enum_lang[stringify!(#var_ident)].clone(),
            });
        } else {
            variants.push(quote::quote! {
                Self::#var_ident(data) => format!("{} {}", data.localize(lang), enum_lang[stringify!(#var_ident)]),
            });
        }
    }

    quote::quote! {
        impl #core_crate::localization::LocalizablePrimitive for #ty {
            fn localize(&self, lang: &#core_crate::localization::LangFile) -> String {
                let enum_lang = &(**lang)[stringify!(#ty)];
                match self {
                    #(#variants)*
                }
            }
        }
    }
    .into()
}
