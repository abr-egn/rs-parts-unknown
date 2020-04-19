use rand::prelude::*;
use serde::{Deserialize, Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::{
    card::Card,
    error::{Error, Result},
    id_map::{Id, IdMap},
    npc::NPC,
    serde_empty,
};

pub type CardId = (Id<Part>, Id<Card>);

#[derive(Debug, Clone)]
pub struct Creature {
    pub parts: IdMap<Part>,
    pub cur_ap: i32,
    pub cur_mp: i32,
    pub dead: bool,
    pub npc: Option<NPC>,
    pub deck: Vec<CardId>,
    pub hand: Vec<CardId>,
    pub discard: Vec<CardId>,
}

impl Creature {
    pub fn new(parts: &[Part], npc: Option<NPC>) -> Self {
        let mut pids = IdMap::new();
        for part in parts {
            let mut tmp = part.clone();
            tmp.cur_hp = tmp.max_hp;
            pids.add(tmp);
        }
        let mut out = Creature {
            parts: pids,
            cur_ap: 0, cur_mp: 0,
            dead: false,
            npc,
            deck: vec![], hand: vec![], discard: vec![],
        };
        out.cur_ap = out.max_ap();
        out.cur_mp = out.max_mp();
        out.reset_cards();
        out
    }

    // Accessors

    pub fn part_cards(&self) -> impl Iterator<Item=(Id<Part>, Id<Card>, &Card)> {
        self.parts.iter()
            .flat_map(|(&id, part)|
                part.cards.iter()
                    .map(move |(&cid, card)| (id, cid, card))
            )
    }

    pub fn max_ap(&self) -> i32 {
        self.parts.values()
            .map(|part| part.ap)
            .sum()
    }

    pub fn max_mp(&self) -> i32 {
        self.parts.values()
            .map(|part| part.mp)
            .sum()
    }

    pub fn hit_action(&self, damage: i32) -> CreatureAction {
        let mut rng = thread_rng();
        let part_id = self.parts.keys().choose(&mut rng).unwrap();
        CreatureAction::ToPart {
            id: *part_id,
            action: PartAction::Hit { damage },
        }
    }

    // Mutators

    pub fn resolve(&mut self, action: &CreatureAction) -> Result<Vec<CreatureEvent>> {
        if self.dead { return Err(Error::DeadCreature); }
        use CreatureAction::*;
        use CreatureEvent::*;
        match *action {
            GainAP { ap } => {
                self.cur_ap += ap;
                return Ok(vec![ChangeAP { delta: ap }]);
            }
            SpendAP { ap } => {
                if self.cur_ap < ap { return Err(Error::NotEnough); }
                self.cur_ap -= ap;
                return Ok(vec![ChangeAP { delta: -ap }]);
            }
            GainMP { mp } => {
                self.cur_mp += mp;
                return Ok(vec![ChangeMP { delta: mp }]);
            }
            SpendMP { mp } => {
                if self.cur_mp < mp { return Err(Error::NotEnough); }
                self.cur_mp -= mp;
                return Ok(vec![ChangeMP { delta: -mp }]);
            }
            ToPart { id, ref action } => {
                let part = self.parts.get_mut(&id).ok_or(Error::NoSuchPart)?;
                let mut self_died = false;
                let out = part.resolve(action).map(|pevs| {
                    let died = pevs.iter().any(|pev| *pev == PartEvent::Died);
                    let mut out: Vec<_> = pevs.into_iter().map(|pev| CreatureEvent::OnPart { id, event: pev }).collect();
                    if died && part.vital && !self_died {
                        self_died = true;
                        out.push(CreatureEvent::Died);
                    }
                    out
                });
                if self_died { self.dead = true; }
                out
            }
        }
    }

    pub fn reset_cards(&mut self) {
        self.deck = self.parts.iter()
            .flat_map(|(&id, part)|
                part.cards.keys()
                    .map(move |&cid| (id, cid))
            ).collect();
        self.hand = vec![];
        self.discard = vec![];
    }

    // TODO: more fine-grained access
    pub fn npc_mut(&mut self) -> Option<&mut NPC> { self.npc.as_mut() }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TsData)]
pub enum CreatureAction {
    GainAP { ap: i32 },
    SpendAP { ap: i32 },
    GainMP { mp: i32 },
    SpendMP { mp: i32 },
    ToPart { id: Id<Part>, action: PartAction }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TsData)]
pub enum CreatureEvent {
    ChangeAP { delta: i32 },
    ChangeMP { delta: i32 },
    OnPart { id: Id<Part>, event: PartEvent },
    #[serde(with = "serde_empty")]
    Died,
}

#[derive(Debug, Clone)]
pub struct Part {
    pub name: String,
    pub cards: IdMap<Card>,
    pub ap: i32,
    pub mp: i32,
    pub max_hp: i32,
    pub cur_hp: i32,
    pub vital: bool,
    pub broken: bool,
    /* TODO: remaining part attributes
    power: i32,
    capacity: i32,
    tags: HashSet<PartTag>,
    joints: Vec<Joint>,
    */
}

impl Part {
    pub fn resolve(&mut self, action: &PartAction) -> Result<Vec<PartEvent>> {
        if self.broken { return Err(Error::BrokenPart); }
        use PartAction::*;
        match *action {
            Hit { damage } => {
                let damage = std::cmp::min(self.cur_hp, damage);
                if damage <= 0 { return Ok(vec![]); }
                self.cur_hp -= damage;
                let mut out = vec![PartEvent::ChangeHP { delta: -damage }];
                if self.cur_hp <= 0 {
                    out.push(PartEvent::Died);
                }
                return Ok(out);
            }
        }
    }
}

impl Default for Part {
    fn default() -> Self {
        Part {
            name: "[DEFAULT]".into(),
            cards: IdMap::new(),
            ap: 0, mp: 0,
            max_hp: 1, cur_hp: 1,
            vital: false, broken: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TsData)]
pub enum PartAction {
    Hit { damage: i32 }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TsData)]
pub enum PartEvent {
    ChangeHP { delta: i32 },
    #[serde(with = "serde_empty")]
    Died,
}

/*
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PartTag {
    Head, Torso, Limb, Arm, Leg,
    Flesh, Metal, Eldritch,
}

#[derive(Debug, Clone)]
pub struct Joint {
    required: HashSet<PartTag>,
    attached: Option<Id<Part>>,
}
*/