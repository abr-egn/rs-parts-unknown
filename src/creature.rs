use serde::Serialize;
use crate::{
    card::Card,
    id_map::{Id, IdMap},
};

#[derive(Debug, Clone, Serialize)]
pub struct Creature {
    pub parts: IdMap<Part>,
    pub cur_ap: i32,
    pub cur_mp: i32,
}

impl Creature {
    pub fn new(parts: &[Part]) -> Self {
        let mut pids = IdMap::new();
        for part in parts {
            pids.add(part.clone());
        }
        Creature { parts: pids, cur_ap: 0, cur_mp: 0 }
    }

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

    pub fn spend_ap(&mut self, ap: i32) -> bool {
        if ap > self.cur_ap { return false; }
        self.cur_ap -= ap;
        true
    }

    pub fn spend_mp(&mut self, mp: i32) -> bool {
        if mp > self.cur_mp { return false; }
        self.cur_mp -= mp;
        true
    }
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
    /*
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