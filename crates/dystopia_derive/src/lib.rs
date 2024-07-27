mod as_built_component;
mod helper;
mod unit;

#[proc_macro_derive(Unit, attributes(si, conversion, conv_method, precision))]
pub fn derive_unit(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    unit::expand_unit_derive(syn::parse(input).unwrap())
}

#[proc_macro_derive(AsBuiltComponent)]
pub fn derive_as_built_component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    as_built_component::expand_as_built_component_derive(syn::parse(input).unwrap())
}
