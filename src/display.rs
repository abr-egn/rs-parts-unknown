use hex;
use js_sys::Array;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::event;

#[wasm_bindgen]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Hex {
    #[wasm_bindgen(readonly)]
    pub x: i32,
    #[wasm_bindgen(readonly)]
    pub y: i32,
}

impl Hex {
    pub fn new(source: &hex::Hex) -> Self {
        Hex { x: source.x, y: source.y }
    }
}

#[wasm_bindgen]
impl Hex {
    #[wasm_bindgen(constructor)]
    pub fn make(x: i32, y: i32) -> Self {
        Hex { x, y }
    }
}

#[wasm_bindgen]
pub struct Event {
    wrapped: event::Meta<event::Event>,
}

impl Event {
    pub fn new(wrapped: event::Meta<event::Event>) -> Self {
        Event { wrapped }
    }
}

#[wasm_bindgen]
impl Event {
    #[wasm_bindgen(getter)]
    pub fn tags(&self) -> Array /* string[] */ {
        self.wrapped.tags.iter()
            .map(|s| JsValue::from(s))
            .collect()
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> JsValue {
        to_value(&self.wrapped.data).unwrap()
    }
}