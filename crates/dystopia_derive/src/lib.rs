mod helper;
mod unit;

#[proc_macro_derive(Unit, attributes(si, conversion, conv_method, precision))]
pub fn derive_unit(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    unit::expand_unit_derive(syn::parse(input).unwrap())
}
