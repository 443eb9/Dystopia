const LANG_SKIP_ATTR: &str = "lang_skip";

pub fn expand_localizable_struct(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = &input.ident;
    let core_crate = crate::helper::core_crate();
    let data = crate::helper::unpack_data_struct(&input.data);
    let mut fields = Vec::with_capacity(data.fields.len());

    for field in &data.fields {
        if field
            .attrs
            .iter()
            .find(|attr| attr.path().get_ident().unwrap() == LANG_SKIP_ATTR)
            .is_some()
        {
            continue;
        }

        let ident = field.ident.as_ref().unwrap();

        fields.push(quote::quote! {
            self.#ident.localize(lang);
        });
    }

    quote::quote! {
        impl #core_crate::localization::LocalizableData for #ty {
            fn localize(&mut self, lang: &#core_crate::localization::LangFile) {
                #(#fields)*
            }
        }

        impl #core_crate::localization::LocalizableData for Option<#ty> {
            fn localize(&mut self, lang: &#core_crate::localization::LangFile) {
                if let Some(data) = self {
                    data.localize(lang);
                }
            }
        }
    }
    .into()
}
