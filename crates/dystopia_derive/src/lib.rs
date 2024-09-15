mod as_built_component;
mod helper;
mod localizable_enum;
mod localizable_struct;
mod quantified;
mod unit;

#[proc_macro_derive(Unit, attributes(si, conversion, conv_method))]
pub fn derive_unit(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    unit::expand_unit_derive(syn::parse(input).unwrap())
}

#[proc_macro_derive(AsBuiltComponent, attributes(share_entity, dynamic_sized))]
pub fn derive_as_built_component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    as_built_component::expand_as_built_component_derive(syn::parse(input).unwrap())
}

#[proc_macro_derive(LocalizableData, attributes(lang_skip))]
pub fn derive_localizable_struct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    localizable_struct::expand_localizable_struct(syn::parse(input).unwrap())
}

#[proc_macro_derive(LocalizableEnum)]
pub fn derive_localizable_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    localizable_enum::expand_localizable_enum(syn::parse(input).unwrap())
}

#[proc_macro_derive(Quantified, attributes(quantify, boundary, min, max))]
pub fn derive_quantified(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quantified::expand_quantified_derive(syn::parse(input).unwrap())
}
