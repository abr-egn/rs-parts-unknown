use hex;
use js_sys::Array;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::card;
use crate::event;
use crate::world::World;

#[wasm_bindgen]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Hex {
    #[wasm_bindgen(readonly)]
    pub x: i32,
    #[wasm_bindgen(readonly)]
    pub y: i32,
}

impl Hex {
    pub fn new(source: hex::Hex) -> Self {
        Hex { x: source.x, y: source.y }
    }
    pub fn old(&self) -> hex::Hex {
        hex::Hex { x: self.x, y: self.y }
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

#[wasm_bindgen]
#[derive(Clone)]
pub struct Behavior {
    wrapped: Box<dyn card::Behavior>,
}

impl Behavior {
    pub fn new(wrapped: Box<dyn card::Behavior>) -> Self {
        Behavior { wrapped }
    }
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl Behavior {
    pub fn highlight(&self, world: &World, cursor: &Hex) -> Array /* Hex[] */ {
        self.wrapped.highlight(world, cursor.old()).into_iter()
            .map(Hex::new)
            .map(JsValue::from)
            .collect()
    }
    pub fn targetValid(&self, world: &World, cursor: &Hex) -> bool {
        self.wrapped.target_valid(world, cursor.old())
    }
    pub fn apply(&self, world: &mut World, target: Hex) -> Array /* Event[] */ {
        self.wrapped.apply(world, target.old()).into_iter()
            .map(Event::new)
            .map(JsValue::from)
            .collect()
    }
}