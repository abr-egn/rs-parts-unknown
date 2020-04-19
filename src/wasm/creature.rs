use std::collections::HashMap;

use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    card, creature,
    id_map::Id,
    npc::{self, ActionKind},
    wasm::{
        to_js_value,
        card::Card,
    },
};

#[derive(Serialize, TsData)]
#[allow(non_snake_case)]
pub struct Creature {
    id: Id<creature::Creature>,
    parts: HashMap<Id<creature::Part>, Part>,
    curAp: i32,
    curMp: i32,
    dead: bool,
    npc: Option<NPC>,
}

impl Creature {
    pub fn new(id: Id<creature::Creature>, source: &creature::Creature) -> Creature {
        let parts = source.parts.iter()
            .map(|(part_id, part)| (*part_id, Part::new(*part_id, id, part)))
            .collect();
        Creature {
            id,
            parts,
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
    ap: i32,
    maxHp: i32,
    curHp: i32,
    dead: bool,
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
            ap: source.ap,
            maxHp: source.max_hp,
            curHp: source.cur_hp,
            dead: source.dead,
        }
    }
}

#[derive(Debug, Serialize, TsData)]
pub struct NPC {
    motion: Option<MotionKind>,
    action: Option<ActionKind>,
}

impl NPC {
    fn new(source: &npc::NPC) -> Self {
        let motion = source.next_motion.as_ref().map(|m| match m {
            npc::Motion::ToMelee => MotionKind::ToMelee,
        });
        NPC {
            motion,
            action: source.next_action.as_ref().map(|a| a.kind.clone()),
        }
    }
}

#[derive(Debug, Serialize, TsData)]
pub enum MotionKind {
    ToMelee,
}