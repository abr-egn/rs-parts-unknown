mod creature;
mod display;
mod id_map;
mod map;
mod player;
mod world;

use hex::Hex;
use wasm_bindgen::prelude::*;

use world::{Event, Meta, World};

pub struct PartsUnknown {
    world: World,
    temp: Option<World>,
}

impl PartsUnknown {
    pub fn new() -> Self {
        PartsUnknown { world: World::new(), temp: None }
    }

    pub fn get_display(&self) -> display::Display {
        display::Display::new(&self.world)
    }

    pub fn move_player(&mut self, x: i32, y: i32) -> Vec<Meta<Event>> {
        self.world.move_player(Hex { x, y })
    }

    pub fn start_check(&mut self) {
        if self.temp.is_some() {
            //godot_error!("start_check during check");
            return
        }
        self.temp = Some(self.world.clone());
        self.world.logging = false;
    }

    pub fn end_check(&mut self) {
        match self.temp.take() {
            Some(old) => self.world = old,
            None => (), //godot_error!("end_check outside check"),
        }
    }
}

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 { a + b }