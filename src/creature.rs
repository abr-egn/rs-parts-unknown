use serde::{Deserialize, Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::{
    card::Card,
    error::{Error, Result},
    id_map::{Id, IdMap},
};

#[derive(Debug, Clone, Serialize)]
pub struct Creature {
    pub parts: IdMap<Part>,
    pub cur_ap: i32,
    pub cur_mp: i32,
    pub dead: bool,
}

impl Creature {
    pub fn new(parts: &[Part]) -> Self {
        let mut pids = IdMap::new();
        for part in parts {
            pids.add(part.clone());
        }
        let mut out = Creature { parts: pids, cur_ap: 0, cur_mp: 0, dead: false };
        out.cur_ap = out.max_ap();
        out.cur_mp = out.max_mp();
        out
    }

    // Accessors

    pub fn cards(&self) -> impl Iterator<Item=(Id<Part>, Id<Card>, &Card)> {
        self.parts.map().iter()
            .flat_map(|(&id, part)|
                part.cards.map().iter()
                    .map(move |(&cid, card)| (id, cid, card))
            )
    }

    pub fn max_ap(&self) -> i32 {
        self.parts.map().values()
            .map(|part| part.ap)
            .sum()
    }

    pub fn max_mp(&self) -> i32 {
        self.parts.map().values()
            .map(|part| part.mp)
            .sum()
    }

    // Mutators

    pub fn resolve(&mut self, action: &CreatureAction) -> Result<Vec<CreatureEvent>> {
        self.check_alive()?;
        use CreatureAction::*;
        use CreatureEvent::*;
        match *action {
            GainAP { ap } => {
                self.cur_ap += ap;
                return Ok(vec![ChangeAP { delta: ap }])
            }
            SpendAP { ap } => {
                if self.cur_ap < ap { return Err(Error::NotEnough); }
                self.cur_ap -= ap;
                return Ok(vec![ChangeAP { delta: -ap }])
            }
            GainMP { mp } => {
                self.cur_mp += mp;
                return Ok(vec![ChangeMP { delta: mp }])
            }
            SpendMP { mp } => {
                if self.cur_mp < mp { return Err(Error::NotEnough); }
                self.cur_mp -= mp;
                return Ok(vec![ChangeMP { delta: -mp }])
            }
        }
    }

    pub fn spend_ap(&mut self, ap: i32) -> Result<()> {
        self.check_alive()?;
        if ap > self.cur_ap { return Err(Error::NotEnough); }
        self.cur_ap -= ap;
        Ok(())
    }

    pub fn spend_mp(&mut self, mp: i32) -> Result<()> {
        self.check_alive()?;
        if mp > self.cur_mp { return Err(Error::NotEnough); }
        self.cur_mp -= mp;
        Ok(())
    }

    fn check_alive(&self) -> Result<()> {
        if self.dead { return Err(Error::DeadCreature); }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TsData)]
pub enum CreatureAction {
    GainAP { ap: i32 },
    SpendAP { ap: i32 },
    GainMP { mp: i32 },
    SpendMP { mp: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TsData)]
pub enum CreatureEvent {
    ChangeAP { delta: i32 },
    ChangeMP { delta: i32 },
}

#[derive(Debug, Clone, Serialize)]
pub struct Part {
    pub name: String,
    pub cards: IdMap<Card>,
    pub ap: i32,
    pub mp: i32,
    pub max_hp: i32,
    pub cur_hp: i32,
    pub vital: bool,
    pub dead: bool,
    /* TODO
    power: i32,
    capacity: i32,
    tags: HashSet<PartTag>,
    joints: Vec<Joint>,
    */
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