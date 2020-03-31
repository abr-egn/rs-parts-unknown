use std::collections::HashMap;

use hex;
use js_sys::Array;
use serde::{Serialize};
use wasm_bindgen::prelude::*;

use crate::creature::Creature;
use crate::id_map::Id;
use crate::map::Tile;
use crate::world::World;

#[wasm_bindgen]
#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Display {
    #[wasm_bindgen(readonly)]
    pub playerId: u32,
    map: Vec<(Hex, Tile)>,
    creatures: HashMap<Id<Creature>, DisplayCreature>,
}

#[wasm_bindgen]
impl Display {
    #[wasm_bindgen(getter)]
    pub fn map(&self) -> Array {
        self.map.iter().map(|(h, _)| JsValue::from(h.clone())).collect()
    }
}

impl Display {
    pub fn new(world: &World) -> Self {
        let player_id = world.player_id();
        let mut creatures = HashMap::new();
        for (id, hex) in world.map().creatures() {
            let label = String::from(if *id == player_id { "P" } else { "X" });
            creatures.insert(*id, DisplayCreature { hex: Hex::new(hex), label });
        }
        Display {
            playerId: player_id.value(),
            map: world.map().tiles().iter()
                .map(|(h, t)| (Hex::new(h), t.clone()))
                .collect(),
            creatures,
        }
    }
}

// Projections

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hex {
    #[wasm_bindgen(readonly)]
    pub x: i32,
    #[wasm_bindgen(readonly)]
    pub y: i32,
}

impl Hex {
    fn new(source: &hex::Hex) -> Self {
        Hex { x: source.x, y: source.y }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DisplayCreature {
    hex: Hex,
    label: String,
}