#![allow(unused)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Card {
    name: String,
    behavior: Box<dyn Behavior>,
}

impl Card {
    pub fn name(&self) -> &str { &self.name }
    pub fn behavior(&self) -> &dyn Behavior { &*self.behavior }
}

#[wasm_bindgen]
impl Card {
    #[wasm_bindgen(js_name = clone)]
    pub fn js_clone(&self) -> Card { self.clone() }
    #[wasm_bindgen(getter = name)]
    pub fn js_name(&self) -> String { self.name.clone() }
}

pub trait Behavior: BehaviorClone {
}

pub trait BehaviorClone {
    fn clone_box(&self) -> Box<dyn Behavior>;
}

impl<T: 'static + Behavior + Clone> BehaviorClone for T {
    fn clone_box(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Behavior> {
    fn clone(&self) -> Self { self.clone_box() }
}