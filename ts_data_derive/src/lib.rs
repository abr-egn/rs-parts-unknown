extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    self,
    parse_macro_input,
    visit::{self, Visit},
};

macro_rules! append {
    ($buf:expr, $($args:tt)*) => {
        ($buf).push_str(&format!($($args)*));
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

fn is_skip(attr: &syn::Attribute) -> bool {
    let name = path_name(&attr.path);
    let tokens = attr.tokens.to_string();
    name == "serde" && tokens == "(skip)"
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
        append!(buffer, "    {}?: ", v.ident);
        match &v.fields {
            syn::Fields::Unit => append!(buffer, "{{}}\n"),
            syn::Fields::Named(n) => {
                append!(buffer, "{{\n");
                build_fields(buffer, "        ", n)?;
                append!(buffer, "    }},\n");
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

fn build_fields(buffer: &mut String, pad: &str, fields: &syn::FieldsNamed) -> Result<(), Error> {
    for v in &fields.named {
        if v.attrs.iter().any(is_skip) { continue; }
        let ident = v.ident.as_ref().expect("field ident");
        let mut trans = TranslateType::new();
        trans.flag_optional = true;
        trans.visit_type(&v.ty);
        if let Some(err) = trans.error {
            return Err(err);
        }
        if trans.out.len() != 1 {
            return Err(Error {
                text: format!("Invalid translated type {:?}", trans.out),
                span: ident.span(),
            })
        }
        append!(buffer, "{}{}", pad, ident);
        if trans.is_optional {
            append!(buffer, "?");
        }
        append!(buffer, ": {},\n", trans.out[0])
    }
    Ok(())
}

struct TranslateType {
    out: Vec<String>,
    flag_optional: bool,
    is_optional: bool,
    error: Option<Error>,
}

impl TranslateType {
    fn new() -> Self {
        TranslateType { out: vec![], flag_optional: false, is_optional: false, error: None }
    }
    fn is_passthrough(s: &str) -> bool {
        match s {
            "Action" => (),
            "ActionKind" => (),
            "Card" => (),
            "Creature" => (),
            "CreatureAction" => (),
            "CreatureEvent" => (),
            "Direction" => (),
            "Event" => (),
            "Hex" => (),
            "Intent" => (),
            "IntentKind" => (),
            "Motion" => (),
            "MotionKind" => (),
            "NPC" => (),
            "Part" => (),
            "PartAction" => (),
            "PartEvent" => (),
            "PartTag" => (),
            "Range" => (),
            "Space" => (),
            "Target" => (),
            "TargetSpec" => (),
            _ => return false,
        }
        true
    }
    fn push_str<S: Into<String>>(&mut self, s: S) {
        self.out.push(s.into());
    }
}

impl<'ast> Visit<'ast> for TranslateType {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let name = path_name(path);
        let span = path.segments.first().expect("first").ident.span();
        match &name as &str {
            // Structural translations
            "Box" => visit::visit_path(self, path),
            "HashMap" => {
                let mut tmp = TranslateType::new();
                visit::visit_path(&mut tmp, path);
                match &tmp.out as &[String] {
                    [k, v] => self.out.push(format!("Map<{}, {}>", k, v)),
                    _ => {
                        self.error = Some(Error {
                            text: format!("invalid HashMap args {:?}", &tmp.out),
                            span,
                        });
                        return;
                    }
                }
            }
            "Id" => {
                let mut tmp = TranslateType::new();
                visit::visit_path(&mut tmp, path);
                match &tmp.out as &[String] {
                    [s] => self.out.push(format!("Id<{}>", s)),
                    _ => {
                        self.error = Some(Error {
                            text: format!("invalid Id args {:?}", &tmp.out),
                            span,
                        });
                        return;
                    }
                }
            }
            "Option" => {
                if self.flag_optional {
                    self.is_optional = true;
                    visit::visit_path(self, path);
                } else {
                    let mut tmp = TranslateType::new();
                    visit::visit_path(&mut tmp, path);
                    match &tmp.out as &[String] {
                        [s] => self.out.push(format!("{} | undefined", s)),
                        _ => {
                            self.error = Some(Error {
                                text: format!("invalid Option args {:?}", &tmp.out),
                                span,
                            });
                            return;
                        }
                    }
                }
            }
            "Vec" => {
                let mut tmp = TranslateType::new();
                visit::visit_path(&mut tmp, path);
                match &tmp.out as &[String] {
                    [s] => self.out.push(format!("{}[]", s)),
                    _ => {
                        self.error = Some(Error {
                            text: format!("invalid Vec args {:?}", &tmp.out),
                            span,
                        });
                        return;
                    }
                }
            }
            // Native types
            "i32" => { self.push_str("number"); }
            "String" => { self.push_str("string"); }
            "ModId" | "TriggerId" => { self.push_str("number"); }
            "bool" => { self.push_str("boolean"); }
            // De-path
            "card::Card" => { self.push_str("Card"); }
            "creature::Creature" => { self.push_str("Creature"); }
            "creature::Part" => { self.push_str("Part"); }
            "hex::Direction" => { self.push_str("Direction"); }
            // Pass through
            s if TranslateType::is_passthrough(s) => {
                self.push_str(s);
            }
            // Error
            _ => {
                self.error = Some(Error {
                    text: format!("unhandled type {}", name),
                    span,
                });
                return;
            },
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
    build_fields(buffer, "    ", fields)?;
    append!(buffer, "}}");
    Ok(())
}

fn path_name(path: &syn::Path) -> String {
    let mut out = String::from(path.segments[0].ident.to_string());
    for s in path.segments.iter().skip(1) {
        out.push_str("::");
        out.push_str(&s.ident.to_string())
    }
    //println!("{:?} / {:?}", out, path.to_token_stream().to_string());
    out
}