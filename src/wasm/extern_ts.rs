use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const EXTERN_TS: &'static str = r#"
export interface Hex {
    x: number,
    y: number,
}

export type Direction = "XY" | "XZ" | "YZ" | "YX" | "ZX" | "ZY";

export type Id<_> = number;
"#;