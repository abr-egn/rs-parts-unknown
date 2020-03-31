mod card;
mod creature;
mod display;
mod id_map;
mod map;
mod world;

use hex::Hex;
use log::{Level, error, info};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use world::World;

#[wasm_bindgen(raw_module = "../ts/for_rust")]
extern "C" {
    fn js_greet(name: &str);
}

#[wasm_bindgen]
pub struct PartsUnknown {
    world: World,
    temp: Option<World>,
}

#[wasm_bindgen]
impl PartsUnknown {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        PartsUnknown { world: World::new(), temp: None }
    }

    #[wasm_bindgen(js_name = buildDisplay)]
    pub fn build_display(&self) -> JsValue {
        to_value(&display::Display::new(&self.world)).unwrap()
    }

    #[wasm_bindgen(js_name = movePlayer)]
    pub fn move_player(&mut self, x: i32, y: i32) -> JsValue {
        to_value(&self.world.move_player(Hex { x, y })).unwrap()
    }

    #[wasm_bindgen(js_name = endTurn)]
    pub fn end_turn(&mut self) -> JsValue {
        to_value(&self.world.end_turn()).unwrap()
    }

    #[wasm_bindgen(js_name = startCheck)]
    pub fn start_check(&mut self) {
        if self.temp.is_some() {
            error!("start_check during check");
            return
        }
        self.temp = Some(self.world.clone());
        self.world.logging = false;
    }

    #[wasm_bindgen(js_name = endCheck)]
    pub fn end_check(&mut self) {
        match self.temp.take() {
            Some(old) => self.world = old,
            None => error!("end_check outside check"),
        }
    }
}

#[wasm_bindgen(start)]
pub fn wasm_start() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Debug).expect("error initializing console_log");

    info!("Parts Unknown WASM initialized.");
    js_greet("User");
}