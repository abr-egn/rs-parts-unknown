use std::collections::HashMap;

use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    card, creature::{self, PartTag},
    id_map::Id,
    npc::{self, Intent, Motion},
    wasm::{
        to_js_value,
        card::Card,
    },
};

#[derive(Serialize, TsData)]
#[allow(non_snake_case)]
pub struct Creature {
    id: Id<creature::Creature>,
    name: String,
    parts: HashMap<Id<creature::Part>, Part>,
    curAp: i32,
    curMp: i32,
    dead: bool,
    npc: Option<NPC>,
    hand: Vec<Card>,
    draw: Vec<Card>,
    discard: Vec<Card>,
}

impl Creature {
    pub fn new(id: Id<creature::Creature>, source: &creature::Creature) -> Creature {
        let parts = source.parts.iter()
            .map(|(part_id, part)| (*part_id, Part::new(*part_id, id, part)))
            .collect();
        let to_card = |&(part_id, card_id)| {
            let card = source.parts.get(part_id).unwrap().cards.get(card_id).unwrap();
            Card::new(card_id, part_id, id, card)
        };
        let hand = source.hand.iter().map(to_card).collect();
        let draw = source.draw.iter().map(to_card).collect();
        let discard = source.discard.iter().map(to_card).collect();
        Creature {
            id,
            name: source.name.clone(),
            parts,
            hand, draw, discard,
            curAp: source.cur_ap,
            curMp: source.cur_mp,
            dead: source.dead,
            npc: source.npc.as_ref().map(NPC::new),
        }
    }
    pub fn js(&self) -> JsValue { to_js_value(&self) }
}

#[derive(Serialize, TsData)]
#[allow(non_snake_case)]
pub struct Part {
    id: Id<creature::Part>,
    creatureId: Id<creature::Creature>,
    name: String,
    cards: HashMap<Id<card::Card>, Card>,
    maxHp: i32,
    curHp: i32,
    thought: i32,
    broken: bool,  // TODO: drop in favor of tags
    tags: Vec<PartTag>,
}

#[allow(non_snake_case)]
impl Part {
    fn new(
        id: Id<creature::Part>,
        creatureId: Id<creature::Creature>,
        source: &creature::Part,
    ) -> Self {
        let cards = source.cards.iter()
            .map(|(&card_id, card)| (card_id, Card::new(card_id, id, creatureId, card)))
            .collect();
        Part {
            id, creatureId, cards,
            name: source.name.clone(),
            thought: source.thought,
            maxHp: source.max_hp,
            curHp: source.cur_hp,
            broken: source.tags().contains(&creature::PartTag::Broken),
            tags: source.tags().into_iter().collect(),
        }
    }
}

#[derive(Debug, Serialize, TsData)]
pub struct NPC {
    motion: Option<Motion>,
    intent: Option<Intent>,
}

impl NPC {
    fn new(source: &npc::NPC) -> Self {
        NPC {
            motion: source.next_motion.clone(),
            intent: source.next_action.clone(),
        }
    }
}