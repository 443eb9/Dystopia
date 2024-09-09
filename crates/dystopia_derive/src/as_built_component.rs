const SHARE_ENTITY: &str = "share_entity";

pub fn expand_as_built_component_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = &input.ident;
    let built_ty = syn::Ident::new(&format!("Built{}", ty), proc_macro2::Span::call_site());

    let data = crate::helper::unpack_data_struct(&input.data);
    let core_crate = crate::helper::core_crate();

    let mut fields = Vec::with_capacity(data.fields.len());
    let mut updates = Vec::with_capacity(data.fields.len());
    let mut from_entities = Vec::with_capacity(data.fields.len());
    let mut shared = 0usize;
    let mut cur_entity = 0usize;

    for field in &data.fields {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let share_entity_attr = field
            .attrs
            .iter()
            .find(|attr| attr.path().get_ident().unwrap() == SHARE_ENTITY);
        let entity_index_override =
            share_entity_attr.map(|e| crate::helper::unpack_list(&e.meta).tokens.clone());

        fields.push(quote::quote! {
            pub #name: bevy::prelude::Entity,
        });

        updates.push(quote::quote! {
            commands.entity(self.#name).insert(
                #core_crate::ui::update::UiDataUpdate::<
                    <#ty as #core_crate::ui::update::AsOriginalComponent>::OriginalComponent,
                    <#ty as #core_crate::ui::update::AsUpdatableData>::UpdatableData,
                >::new(
                    data.#name.clone().into()
                )
            );
        });

        from_entities.push(if let Some(ovrd) = entity_index_override {
            shared += 1;
            quote::quote! {
                #name: entities[#ovrd].clone(),
            }
        } else {
            cur_entity += 1;
            quote::quote! {
                #name: entities[#cur_entity - 1],
            }
        });
    }

    let fields_len = fields.len();

    quote::quote! {
        #[derive(bevy::prelude::Component)]
        pub struct #built_ty {
            #(#fields)*
        }

        impl #built_ty {
            pub fn from_entities(entities: Vec<bevy::prelude::Entity>) -> Self {
                assert_eq!(
                    entities.len(),
                    #fields_len - #shared,
                    "Entities count {} not matching with fields cound {}. (With {} data sharing entities)",
                    entities.len(),
                    #fields_len,
                    #shared,
                );

                Self {
                    #(#from_entities)*
                }
            }

            pub fn update(&self, data: &#ty, commands: &mut bevy::prelude::Commands) {
                #(#updates)*
            }
        }

        impl #core_crate::ui::update::AsBuiltComponent for #ty {
            const NUM_FIELDS: usize = #fields_len;
        }
    }
    .into()
}
