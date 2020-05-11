use serde::{Deserialize, Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    action::Path,
    card, creature,
    id_map::Id,
    part,
    world::World,
};

#[derive(Serialize, Deserialize, TsData)]
#[allow(non_snake_case)]
pub struct Card {
    pub id: Id<card::Card>,
    pub partId: Id<part::Part>,
    pub creatureId: Id<creature::Creature>,
    pub name: String,
    pub apCost: i32,
}

impl Card {
    #[allow(non_snake_case)]
    pub fn new(
        id: Id<card::Card>,
        partId: Id<part::Part>,
        creatureId: Id<creature::Creature>,
        source: &card::Card,
    ) -> Self {
        Card {
            id, partId, creatureId,
            name: source.name.clone(),
            apCost: source.ap_cost,
        }
    }

    pub fn get<'a>(&self, world: &'a World) -> Option<&'a card::Card> {
        let creature = world.creatures().get(self.creatureId)?;
        let part = creature.parts.get(self.partId)?;
        part.cards.get(self.id)
    }

    pub fn source(&self) -> Path {
        Path::Part { cid: self.creatureId, pid: self.partId }
    }
}