extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;

macro_rules! append {
    ($buf:ident, $($args:tt)*) => {
        $buf.push_str(&format!($($args)*));
    }
}

#[proc_macro_derive(TsData)]
pub fn ts_data_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let mut output: String = String::new();
    append!(output, "\n/* Start TsData Generated */\n");
    match ast.data {
        syn::Data::Enum(e) => build_enum(&mut output, &ast.ident, &e),
        _ => unimplemented!(),
    }
    append!(output, "\n/* End TsData Generated */\n");

    let gen = quote! {
        #[wasm_bindgen(typescript_custom_section)]
        const TS_APPEND: &'static str = #output;
    };
    gen.into()
}

fn build_enum(buffer: &mut String, name: &syn::Ident, data: &syn::DataEnum) {
    let all_simple: bool = data.variants.iter()
        .all(|v| match v.fields {
            syn::Fields::Unit => true,
            _ => false,
        });
    if all_simple {
        build_simple_enum(buffer, name, data);
        return;
    }
    append!(buffer, "export interface {} {{\n", name);
    append!(buffer, "}}\n");
}

fn build_simple_enum(buffer: &mut String, name: &syn::Ident, data: &syn::DataEnum) {
    append!(buffer, "export type {} = \"{}\"", name, data.variants[0].ident);
    for v in data.variants.iter().skip(1) {
        append!(buffer, " | \"{}\"", v.ident);
    }
    append!(buffer, ";");
}