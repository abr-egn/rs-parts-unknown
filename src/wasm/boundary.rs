use std::collections::HashSet;

use hex::Hex;
use js_sys::Array;
use serde::Serialize;
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::wasm::{from_js_value, to_js_value};

#[derive(Debug, Clone, Serialize, TsData)]
pub struct Boundary {
    pub hex: Hex,
    pub sides: Vec<hex::Direction>,
}

fn find_boundary(shape: &[Hex]) -> Vec<Boundary> {
    let shape: HashSet<Hex> = shape.into_iter().cloned().collect();
    let mut out = vec![];
    for &hex in &shape {
        let mut bound = Boundary { hex, sides: vec![] };
        for dir in hex::Direction::all() {
            if !shape.contains(&(hex + dir.delta())) {
                bound.sides.push(*dir);
            }
        }
        if !bound.sides.is_empty() {
            out.push(bound);
        }
    }
    out
}

#[allow(unused)]
#[wasm_bindgen(skip_typescript, js_name="findBoundary")]
pub fn js_find_boundary(shape: &Array /* Hex[] */) -> Array /* Boundary[] */ {
    let shape: Vec<Hex> = shape.iter().map(from_js_value).collect();
    find_boundary(&shape).iter().map(to_js_value).collect()
}

#[wasm_bindgen(typescript_custom_section)]
const FIND_BOUNDARY_TS: &'static str = r#"
export function findBoundary(shape: Hex[]): Boundary[];
"#;