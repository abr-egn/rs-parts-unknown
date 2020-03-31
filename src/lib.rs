mod card;
mod creature;
mod display;
mod event;
mod id_map;
mod map;
mod world;

use log::{Level, info};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "../ts/for_rust")]
extern "C" {
    fn js_greet(name: &str);
}

#[wasm_bindgen(start)]
pub fn wasm_start() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Debug).expect("error initializing console_log");

    info!("Parts Unknown WASM initialized.");
    js_greet("User");
}