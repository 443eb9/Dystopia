const MIN_ATTR: &str = "min";
const MAX_ATTR: &str = "max";
const BOUNDARY_ATTR: &str = "boundary";
const QUANTIFY_ATTR: &str = "quantify";

pub fn expand_quantified_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = &input.ident;
    let data = crate::helper::unpack_data_enum(&input.data);
    let core_crate = crate::helper::core_crate();

    let quantify_precision =
        &crate::helper::find_attr_get_meta(&input.attrs, QUANTIFY_ATTR, crate::helper::unpack_list)
            .tokens;
    let min =
        crate::helper::try_find_attr_get_meta(&input.attrs, MIN_ATTR, crate::helper::unpack_list)
            .map(|m| &m.tokens);
    let max =
        crate::helper::try_find_attr_get_meta(&input.attrs, MAX_ATTR, crate::helper::unpack_list)
            .map(|m| &m.tokens);

    let mut quantifying = Vec::with_capacity(data.variants.len());
    let mut sampling = Vec::with_capacity(data.variants.len());
    let n_variants = data.variants.len();
    assert!(
        n_variants > 2,
        "Quantified enums must have at least 2 variants."
    );

    let boundaries = data
        .variants
        .iter()
        .enumerate()
        .filter_map(|(index, variant)| {
            let list = crate::helper::try_find_attr_get_meta(
                &variant.attrs,
                BOUNDARY_ATTR,
                crate::helper::unpack_list,
            );

            if index == 0 {
                if list.is_some() {
                    panic!("`boundary` is not allowed on the first variant.")
                } else {
                    return None;
                }
            }

            Some(
                &list
                    .expect("Expected `boundary` attribute on each variant except the first one.")
                    .tokens,
            )
        })
        .collect::<Vec<_>>();

    for (i_variant, variant) in data.variants.iter().enumerate() {
        let ident = &variant.ident;

        if i_variant == 0 {
            let boundary = boundaries[0];
            if let Some(min) = min {
                quantifying.push(quote::quote! {
                    if value > #min && value < #boundary {
                        Self::#ident
                    }
                });
                sampling.push(quote::quote! {
                    Self::#ident => rng.gen_range(#min..#boundary),
                });
            } else {
                quantifying.push(quote::quote! {
                    if value < #boundary {
                        Self::#ident
                    }
                });
                sampling.push(quote::quote! {
                    Self::#ident => panic!("To sample the first variant, you need to specify `min` attribute."),
                });
            };
        } else if i_variant == n_variants - 1 {
            if let Some(max) = max {
                quantifying.push(quote::quote! {
                    else if value < #max {
                        Self::#ident
                    } else {
                        panic!("Value out of range.")
                    }
                });

                let boundary = boundaries[n_variants - 2];
                sampling.push(quote::quote! {
                    Self::#ident => rng.gen_range(#boundary..#max),
                });
            } else {
                quantifying.push(quote::quote! {
                    else {
                        Self::#ident
                    }
                });
                sampling.push(quote::quote! {
                    Self::#ident => panic!("To sample the last variant, you need to specify `max` attribute."),
                });
            };
        } else {
            let lower = boundaries[i_variant - 1];
            let upper = boundaries[i_variant];
            quantifying.push(quote::quote! {
                else if value > #lower && value > #upper {
                    Self::#ident
                }
            });
            sampling.push(quote::quote! {
                Self::#ident => rng.gen_range(#lower..#upper),
            });
        }
    }

    quote::quote! {
        impl #core_crate::sci::Quantified<#quantify_precision> for #ty {
            fn quantify(value: #quantify_precision) -> Self {
                #(#quantifying)*
            }

            fn sample(self, rng: &mut impl rand::Rng) -> #quantify_precision {
                match self {
                    #(#sampling)*
                }
            }
        }
    }
    .into()
}
