pub fn expand_as_built_component_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = &input.ident;
    let built_ty = syn::Ident::new(&format!("Built{}", ty), proc_macro2::Span::call_site());

    let data = match &input.data {
        syn::Data::Struct(data) => data,
        syn::Data::Enum(_) | syn::Data::Union(_) => {
            panic!("AsBuiltComponent can only be derived for structs.")
        }
    };

    let core_crate = crate::helper::core_crate();

    let mut fields = Vec::with_capacity(data.fields.len());

    for field in &data.fields {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        fields.push(quote::quote! {
            pub #name: <#ty as #core_crate::ui::primitive::PrimitveUiData>::BuiltType,
        });
    }

    quote::quote! {
        #[derive(bevy::prelude::Component)]
        pub struct #built_ty {
            #(#fields)*
        }

        impl #core_crate::ui::primitive::AsBuiltComponent for #built_ty {}
    }
    .into()
}
