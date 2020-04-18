use serde::{Deserialize, Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    card, creature,
    id_map::Id,
};

#[derive(Serialize, Deserialize, TsData)]
#[allow(non_snake_case)]
pub struct Card {
    pub id: Id<card::Card>,
    pub partId: Id<creature::Part>,
    pub creatureId: Id<creature::Creature>,
    pub name: String,
    pub apCost: i32,
}

impl Card {
    #[allow(non_snake_case)]
    pub fn new(
        id: Id<card::Card>,
        partId: Id<creature::Part>,
        creatureId: Id<creature::Creature>,
        source: &card::Card,
    ) -> Self {
        Card {
            id, partId, creatureId,
            name: source.name.clone(),
            apCost: source.ap_cost,
        }
    }
}