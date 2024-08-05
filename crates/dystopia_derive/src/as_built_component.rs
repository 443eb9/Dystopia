pub fn expand_as_built_component_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = &input.ident;
    let built_ty = syn::Ident::new(&format!("Built{}", ty), proc_macro2::Span::call_site());

    let data = crate::helper::unpack_data_struct(&input.data);
    let core_crate = crate::helper::core_crate();

    let mut fields = Vec::with_capacity(data.fields.len());
    let mut updates = Vec::with_capacity(data.fields.len());
    let mut from_entities = Vec::with_capacity(data.fields.len());

    for (i_field, field) in data.fields.iter().enumerate() {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        fields.push(quote::quote! {
            pub #name: bevy::prelude::Entity,
        });

        updates.push(quote::quote! {
            commands.entity(self.#name).insert(
                #core_crate::ui::primitive::PrimitiveDataUpdate::<
                    <#ty as #core_crate::ui::primitive::AsUiComponent>::UiComponent
                > {
                    new: (&data.#name).into(),
                }
            );
        });

        from_entities.push(quote::quote! {
            #name: entities[#i_field],
        });
    }

    let fields_len = fields.len();

    quote::quote! {
        #[derive(bevy::prelude::Component)]
        pub struct #built_ty {
            #(#fields)*
        }

        impl #built_ty {
            pub fn from_entities(entities: Vec<Entity>) -> Self {
                assert_eq!(
                    entities.len(),
                    #fields_len,
                    "Entities count {} not matching with fields cound {}",
                    entities.len(),
                    #fields_len
                );

                Self {
                    #(#from_entities)*
                }
            }

            pub fn update(&self, data: &#ty, commands: &mut bevy::prelude::Commands) {
                #(#updates)*
            }
        }

        impl #core_crate::ui::primitive::AsBuiltComponent for #ty {
            const NUM_FIELDS: usize = #fields_len;
        }
    }
    .into()
}
