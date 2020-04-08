extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{self, visit_mut::{VisitMut, visit_derive_input_mut}};

macro_rules! append {
    ($buf:ident, $($args:tt)*) => {
        $buf.push_str(&format!($($args)*));
    }
}

#[proc_macro_derive(TsData)]
pub fn ts_data_derive(input: TokenStream) -> TokenStream {
    let mut ast: syn::DeriveInput = syn::parse(input).unwrap();
    visit_derive_input_mut(&mut TypeMapper {}, &mut ast);

    let mut output: String = String::new();
    append!(output, "\n/* TsData Generated */\n");
    match ast.data {
        syn::Data::Enum(e) => build_enum(&mut output, &ast.ident, &e),
        syn::Data::Struct(s) => build_struct(&mut output, &ast.ident, &s),
        _ => panic!("unhandled ast.data branch in {}", &ast.ident),
    }
    //append!(output, "\n");

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
        if let Some(ty) = extract_generic("Option", &v.ty) {
            append!(buffer, "    {}?: {},\n", ident, quote! { #ty });
        } else if let Some(ty) = extract_generic("Vec", &v.ty) {
            append!(buffer, "    {}: {}[],\n", ident, quote! { #ty });
        } else {
            let ty = &v.ty;
            append!(buffer, "    {}: {},\n", ident, quote! { #ty });
        }
    }
    append!(buffer, "}}");
}

fn extract_generic<'a>(name: &str, ty: &'a syn::Type) -> Option<&'a syn::Type> {
    let path = match ty {
        syn::Type::Path(p) => &p.path,
        _ => return None,
    };
    if path.segments.len() != 1 {
        return None;
    }
    let segment = &path.segments[0];
    if segment.ident.to_string() != name {
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

struct TypeMapper {}

impl VisitMut for TypeMapper {
    fn visit_attribute_mut(&mut self, _: &mut syn::Attribute) { }
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        let name = path_name(path);
        match &name as &str {
            // Pass through
            "Card" => (),
            "Creature" => (),
            "Direction" => (),
            "Hex" => (),
            "Id" => (),
            "Option" => (),
            "Part" => (),
            "Space" => (),
            "Vec" => (),
            // De-path
            "card::Card" => replace_all(path, "Card"),
            "creature::Creature" => replace_all(path, "Creature"),
            "creature::Part" => replace_all(path, "Part"),
            "hex::Direction" => replace_all(path, "Direction"),
            // Native types
            "HashMap" => replace_first(path, "Map"),
            "i32" => replace_first(path, "number"),
            "String" => replace_first(path, "string"),

            _ => panic!("unhandled type {} : {:?}", name, path),
        }
        for s in &mut path.segments {
            self.visit_path_segment_mut(s);
        }
    }
}

fn replace_first(path: &mut syn::Path, ident: &str) {
    path.segments[0].ident = syn::Ident::new(ident, path.segments[0].ident.span());
}

fn replace_all(path: &mut syn::Path, ident: &str) {
    let mut segments = syn::punctuated::Punctuated::new();
    segments.push(syn::PathSegment {
        ident: syn::Ident::new(ident, path.segments[0].ident.span()),
        arguments: syn::PathArguments::None,
    });
    path.segments = segments;
}

fn path_name(path: &syn::Path) -> String {
    let mut out = String::from(path.segments[0].ident.to_string());
    for s in path.segments.iter().skip(1) {
        out.push_str("::");
        out.push_str(&s.ident.to_string())
    }
    out
}