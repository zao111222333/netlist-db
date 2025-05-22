use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};
mod builder;

#[proc_macro_derive(Builder, attributes(netlist))]
pub fn macro_group(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let toks = builder::inner(&ast).unwrap_or_else(|err| err.to_compile_error());
    toks.into()
}
