use serde::{Serialize, de::DeserializeOwned};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::JsValue;

mod behavior;
mod boundary;
mod card;
mod creature;
mod extern_ts;
mod world;

fn to_js_value<T: Serialize>(t: &T) -> JsValue { to_value(t).unwrap() }
fn from_js_value<T: DeserializeOwned>(js: JsValue) -> T { 
    from_value(js).unwrap()
}