extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    self,
    parse_macro_input,
    visit_mut::{VisitMut, visit_derive_input_mut},
};

macro_rules! append {
    ($buf:ident, $($args:tt)*) => {
        $buf.push_str(&format!($($args)*));
    }
}

#[proc_macro_derive(TsData)]
pub fn ts_data_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    let tokens = match derive_impl(ast) {
        Ok(output) => {
            quote! {
                #[wasm_bindgen(typescript_custom_section)]
                const TS_APPEND: &'static str = #output;
            }
        }
        Err(e) => {
            let text = e.text;
            quote_spanned! {e.span=>
                compile_error!(#text);
            }
        }
    };
    tokens.into()
}

fn derive_impl(ast: syn::DeriveInput) -> Result<String, Error> {
    let mut ast = ast;
    let mut mapper = TypeMapper::new();
    visit_derive_input_mut(&mut mapper, &mut ast);
    if let Some(e) = mapper.error { return Err(e); }

    let mut output: String = String::new();
    append!(output, "\n/* TsData Generated */\n");
    match ast.data {
        syn::Data::Enum(e) => build_enum(&mut output, &ast.ident, &e)?,
        syn::Data::Struct(s) => build_struct(&mut output, &ast.ident, &s)?,
        _ => return Err(Error {
            text: String::from("unhandled ast.data branch"),
            span: ast.ident.span(),
        }),
    }
    Ok(output)
}

struct Error {
    text: String,
    span: proc_macro2::Span,
}

fn build_enum(buffer: &mut String, name: &syn::Ident, data: &syn::DataEnum) -> Result<(), Error> {
    let all_simple: bool = data.variants.iter()
        .all(|v| v.fields == syn::Fields::Unit);
    if all_simple {
        build_simple_enum(buffer, name, data);
        return Ok(());
    }
    append!(buffer, "export interface {} {{\n", name);
    for v in &data.variants {
        match &v.fields {
            syn::Fields::Unit => append!(buffer, "    {}: boolean | undefined,\n", v.ident),
            syn::Fields::Named(n) => {
                append!(buffer, "    {}: {{\n", v.ident);
                build_fields(buffer, "        ", n);
                append!(buffer, "    }} | undefined,\n");
            }
            v => return Err(Error {
                text: format!("unhandled enum variant: {:?}", v),
                span: name.span(),
            })
        }
    }
    append!(buffer, "}}\n");
    Ok(())
}

fn build_simple_enum(buffer: &mut String, name: &syn::Ident, data: &syn::DataEnum) {
    append!(buffer, "export type {} = \"{}\"", name, data.variants[0].ident);
    for v in data.variants.iter().skip(1) {
        append!(buffer, " | \"{}\"", v.ident);
    }
    append!(buffer, ";");
}

fn build_fields(buffer: &mut String, pad: &str, fields: &syn::FieldsNamed) {
    for v in &fields.named {
        let ident = v.ident.as_ref().expect("field ident");
        if let Some(ty) = extract_generic("Option", &v.ty) {
            append!(buffer, "{}{}?: {},\n", pad, ident, quote! { #ty });
        } else if let Some(ty) = extract_generic("Vec", &v.ty) {
            append!(buffer, "{}{}: {}[],\n", pad, ident, quote! { #ty });
        } else if let Some(ty) = extract_generic("Box", &v.ty) {
            append!(buffer, "{}{}: {},\n", pad, ident, quote! { #ty });
        } else {
            let ty = &v.ty;
            append!(buffer, "{}{}: {},\n", pad, ident, quote! { #ty });
        }
    }
}

fn build_struct(buffer: &mut String, name: &syn::Ident, data: &syn::DataStruct) -> Result<(), Error> {
    append!(buffer, "export interface {} {{\n", name);
    let fields = match &data.fields {
        syn::Fields::Named(f) => f,
        _ => return Err(Error {
            text: format!("unhandled Fields branch: {:?}", data.fields),
            span: name.span(),
        })
    };
    build_fields(buffer, "    ", fields);
    append!(buffer, "}}");
    Ok(())
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

struct TypeMapper {
    error: Option<Error>,
}

impl TypeMapper {
    fn new() -> Self { TypeMapper { error: None }}
}

impl VisitMut for TypeMapper {
    fn visit_attribute_mut(&mut self, _: &mut syn::Attribute) { }
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if self.error.is_some() { return; }
        let name = path_name(path);
        let span = path.segments.first().expect("first").ident.span();
        match &name as &str {
            // Pass through
            "Action" => (),
            "Box" => (),
            "Card" => (),
            "Creature" => (),
            "Direction" => (),
            "Event" => (),
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
            "ModId" | "TriggerId" => replace_first(path, "number"),

            _ => {
                self.error = Some(Error {
                    text: format!("unhandled type {}", name),
                    span,
                });
                return;
            },
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