use std::collections::HashMap;

use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    card, creature::{self, PartTag},
    id_map::Id,
    npc::{self, IntentKind},
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
}

impl Creature {
    pub fn new(id: Id<creature::Creature>, source: &creature::Creature) -> Creature {
        let parts = source.parts.iter()
            .map(|(part_id, part)| (*part_id, Part::new(*part_id, id, part)))
            .collect();
        let hand = source.hand.iter()
            .map(|&(part_id, card_id)| {
                let card = source.parts.get(part_id).unwrap().cards.get(card_id).unwrap();
                Card::new(card_id, part_id, id, card)
            })
            .collect();
        Creature {
            id,
            name: source.name.clone(),
            parts,
            hand,
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
            broken: source.tags.contains(&creature::PartTag::Broken),
            tags: source.tags.iter().cloned().collect(),
        }
    }
}

#[derive(Debug, Serialize, TsData)]
pub struct NPC {
    motion: Option<MotionKind>,
    intent: Option<IntentKind>,
}

impl NPC {
    fn new(source: &npc::NPC) -> Self {
        let motion = source.next_motion.as_ref().map(|m| match m {
            npc::Motion::ToMelee => MotionKind::ToMelee,
        });
        NPC {
            motion,
            intent: source.next_action.as_ref().map(|a| a.kind.clone()),
        }
    }
}

#[derive(Debug, Serialize, TsData)]
pub enum MotionKind {
    ToMelee,
}