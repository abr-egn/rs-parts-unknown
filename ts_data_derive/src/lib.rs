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
    append!(output, "\n/* TsData Generated */\n");
    match ast.data {
        syn::Data::Enum(e) => build_enum(&mut output, &ast.ident, &e),
        syn::Data::Struct(s) => build_struct(&mut output, &ast.ident, &s),
        _ => panic!("unhandled ast.data branch in {}", &ast.ident),
    }
    append!(output, "\n");

    let gen = quote! {
        #[wasm_bindgen(typescript_custom_section)]
        const TS_APPEND: &'static str = #output;
    };
    gen.into()
}

fn build_enum(buffer: &mut String, name: &syn::Ident, data: &syn::DataEnum) {
    let all_simple: bool = data.variants.iter()
        .all(|v| v.fields == syn::Fields::Unit);
    if all_simple {
        build_simple_enum(buffer, name, data);
        return;
    }
    // TODO
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

fn build_struct(buffer: &mut String, name: &syn::Ident, data: &syn::DataStruct) {
    append!(buffer, "export interface {} {{\n", name);
    let fields = match &data.fields {
        syn::Fields::Named(f) => f,
        _ => panic!("unhandled Fields branch in {}: {:?}", name, data.fields),
    };
    for v in &fields.named {
        let ident = v.ident.as_ref().expect("struct field ident");
        if let Some(ty) = extract_option(&v.ty) {
            append!(buffer, "    {}?: {},\n", ident, quote! { #ty })
        } else {
            let ty = &v.ty;
            append!(buffer, "    {}: {},\n", ident, quote! { #ty });
        }
    }
    append!(buffer, "}}");
}

fn extract_option(ty: &syn::Type) -> Option<&syn::Type> {
    let path = match ty {
        syn::Type::Path(p) => &p.path,
        _ => return None,
    };
    if path.segments.len() != 1 {
        return None;
    }
    let segment = &path.segments[0];
    if segment.ident.to_string() != "Option" {
        return None;
    }
    let args = match &segment.arguments {
        syn::PathArguments::AngleBracketed(ab) => &ab.args,
        _ => return None,
    };
    if args.len() != 1 {
        return None;
    }
    match &args[0] {
        syn::GenericArgument::Type(t) => return Some(t),
        _ => return None,
    }
}