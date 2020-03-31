use std::collections::HashMap;

use hex::Hex;
use serde::{Serialize};

use crate::creature::Creature;
use crate::id_map::Id;
use crate::map::Tile;
use crate::world::World;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Display {
    map: Vec<(Hex, Tile)>,
    player_id: Id<Creature>,
    creatures: HashMap<Id<Creature>, DisplayCreature>,
}

impl Display {
    pub fn new(world: &World) -> Self {
        let player_id = world.player_id();
        let mut creatures = HashMap::new();
        for (id, &hex) in world.map().creatures() {
            let label = String::from(if *id == player_id { "P" } else { "X" });
            creatures.insert(*id, DisplayCreature { hex, label });
        }
        Display {
            map: world.map().tiles().iter()
                .map(|(h, t)| (h.clone(), t.clone()))
                .collect(),
            player_id,
            creatures,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DisplayCreature {
    hex: Hex,
    label: String,
}