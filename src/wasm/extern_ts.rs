use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const EXTERN_TS: &'static str = r#"
export interface Hex {
    x: number,
    y: number,
}

export type Direction = "XY" | "XZ" | "YZ" | "YX" | "ZX" | "ZY";

// This makes Id<T> nominal rather than structural; without it,
// Id<foo> would be treated as equal to Id<bar>.
declare const phantom: unique symbol;
export type Id<T> = number & { [phantom]?: T }

export type TagMod = {};

export type Status = {};
"#;