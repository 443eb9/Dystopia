const SHARE_ENTITY: &str = "share_entity";
const DYNAMIC_SIZED: &str = "dynamic_sized";

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
    let mut n_reserved = 0usize;
    let mut n_entities = 0usize;

    for (i_field, field) in data.fields.iter().enumerate() {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let share_entity_attr = field
            .attrs
            .iter()
            .find(|attr| attr.path().get_ident().unwrap() == SHARE_ENTITY);
        let dynamic_sized_attr = field
            .attrs
            .iter()
            .find(|attr| attr.path().get_ident().unwrap() == DYNAMIC_SIZED);
        let is_dynamic_sized = dynamic_sized_attr.is_some();
        if dynamic_sized_attr.is_some() && i_field == data.fields.len() {
            panic!("`dynamic_sized` field must at the back of struct.")
        }

        let reserved = dynamic_sized_attr
            .map(|a| {
                let size = crate::helper::unpack_list(&a.meta).tokens.to_string();
                usize::from_str_radix(&size, 10).expect(&format!("Invalid reserve size {}.", size))
            })
            .unwrap_or_default();

        let entity_index_override =
            share_entity_attr.map(|e| crate::helper::unpack_list(&e.meta).tokens.clone());

        fields.push(if is_dynamic_sized {
            quote::quote! {
                pub #name: <#ty as #core_crate::ui::update::DynamicSizedUpdatableData>::Container,
            }
        } else {
            quote::quote! {
                pub #name: bevy::prelude::Entity,
            }
        });

        updates.push(if is_dynamic_sized {
            quote::quote! {
                <#ty as #core_crate::ui::update::DynamicSizedUpdatableData>::iterate(&data.#name)
                    .zip(self.#name.iter().take(<#ty as #core_crate::ui::update::DynamicSizedUpdatableData>::len(&data.#name)))
                    .for_each(|(data, entity)| {
                        commands.entity(*entity).insert(
                            #core_crate::ui::update::UiDataUpdate::<
                                <<#ty as #core_crate::ui::update::DynamicSizedUpdatableData>::Element as #core_crate::ui::update::AsOriginalComponent>::OriginalComponent,
                                <<#ty as #core_crate::ui::update::DynamicSizedUpdatableData>::Element as #core_crate::ui::update::AsUpdatableData>::UpdatableData,
                            >::new(
                                data.clone().into()
                            )
                        );
                    });
            }
        } else {
            quote::quote! {
                commands.entity(self.#name).insert(
                    #core_crate::ui::update::UiDataUpdate::<
                        <#ty as #core_crate::ui::update::AsOriginalComponent>::OriginalComponent,
                        <#ty as #core_crate::ui::update::AsUpdatableData>::UpdatableData,
                    >::new(
                        data.#name.clone().into()
                    )
                );
            }
        });

        from_entities.push(if is_dynamic_sized {
            cur_entity += reserved as usize;
            n_entities += reserved;
            n_reserved += reserved;
            quote::quote! {
                #name: entities[#cur_entity - #reserved..#cur_entity].to_vec(),
            }
        } else if let Some(ovrd) = entity_index_override {
            shared += 1;
            quote::quote! {
                #name: entities[#ovrd].clone(),
            }
        } else {
            cur_entity += 1;
            n_entities += 1;
            quote::quote! {
                #name: entities[#cur_entity - 1],
            }
        });
    }

    let n_fields = fields.len();

    quote::quote! {
        #[derive(bevy::prelude::Component)]
        pub struct #built_ty {
            #(#fields)*
        }

        impl #built_ty {
            pub fn from_entities(entities: Vec<bevy::prelude::Entity>) -> Self {
                assert_eq!(
                    entities.len(),
                    #n_entities,
                    "Entities count {} not matching with fields cound {}. With {} data sharing entities, and {} entities reserved.",
                    entities.len(),
                    #n_entities,
                    #shared,
                    #n_reserved
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
            const NUM_FIELDS: usize = #n_fields;
        }
    }
    .into()
}
