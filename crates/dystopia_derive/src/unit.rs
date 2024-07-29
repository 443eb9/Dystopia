const SI_ATTR: &'static str = "si";
const CONVERSION_ATTR: &'static str = "conversion";
const CONV_METHOD_ATTR: &'static str = "conv_method";
const PRECISION_ATTR: &'static str = "precision";

pub fn expand_unit_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = &input.ident;
    let precision_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().get_ident().unwrap() == PRECISION_ATTR)
        .unwrap_or_else(|| panic!("You need to specify a precision for the SI unit, like f64."));
    let presion = &crate::helper::unpack_list(&precision_attr.meta).tokens;

    let data = crate::helper::unpack_data_enum(&input.data);

    let mut si = None;
    let mut conversions = Vec::with_capacity(data.variants.len());

    for (i_variant, variant) in data.variants.iter().enumerate() {
        let var_ident = &variant.ident;

        if variant
            .attrs
            .iter()
            .find(|attr| attr.path().get_ident().unwrap() == SI_ATTR)
            .is_some()
        {
            si = Some(i_variant);
            conversions.push(quote::quote! {
                Self::#var_ident(t) => t,
            });

            continue;
        }

        let conv_attr = variant
            .attrs
            .iter()
            .find(|attr| attr.path().get_ident().unwrap() == CONVERSION_ATTR)
            .unwrap_or_else(|| panic!("Besides the SI unit, all units must specify their conversion factors to SI unit."));

        let conv_factor = &crate::helper::unpack_name_value(&conv_attr.meta).value;

        let conv_method = variant
            .attrs
            .iter()
            .find(|attr| attr.path().get_ident().unwrap() == CONV_METHOD_ATTR)
            .map(|a| {
                let syn::Expr::Lit(expr) = &crate::helper::unpack_name_value(&a.meta).value else {
                    unreachable!()
                };
                let syn::Lit::Str(s) = &expr.lit else {
                    unreachable!()
                };
                s.value()
            })
            .unwrap_or("mul".to_string());

        conversions.push(match conv_method.as_str() {
            "add" => quote::quote! {
                Self::#var_ident(t) => t + #conv_factor,
            },
            "sub" => quote::quote! {
                Self::#var_ident(t) => t - #conv_factor,
            },
            "mul" => quote::quote! {
                Self::#var_ident(t) => t * #conv_factor,
            },
            "div" => quote::quote! {
                Self::#var_ident(t) => t / #conv_factor,
            },
            _ => panic!(
                "Invalid method: {}. conv_method must be one of the following: add, sub, mul, div.",
                { conv_method }
            ),
        });
    }

    let si = &data.variants[si.unwrap_or_else(|| panic!("You have to specify a SI unit."))].ident;

    quote::quote! {
        impl Unit for #ty {
            type Precision = #presion;

            fn to_si(self) -> Self::Precision {
                match self {
                    #(#conversions)*
                }
            }

            fn to_si_unit(self) -> Self {
                Self::#si(self.to_si())
            }
        }
    }
    .into()
}
