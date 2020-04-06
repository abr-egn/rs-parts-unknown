use serde::Serialize;
use crate::{
    card::Card,
    id_map::{Id, IdMap},
};

#[derive(Debug, Clone, Serialize)]
pub struct Creature {
    pub kind: Kind,
    pub parts: IdMap<Part>,
    pub cur_ap: i32,
}

impl Creature {
    pub fn new(kind: Kind, parts: &[Part]) -> Self {
        let mut pids = IdMap::new();
        for part in parts {
            pids.add(part.clone());
        }
        Creature { kind, parts: pids, cur_ap: 0 }
    }

    pub fn kind(&self) -> &Kind { &self.kind }

    pub fn parts(&self) -> &IdMap<Part> { &self.parts }

    pub fn cards(&self) -> impl Iterator<Item=(Id<Part>, Id<Card>, &Card)> {
        self.parts.map().iter()
            .flat_map(|(&id, part)|
                part.cards.map().iter()
                    .map(move |(&cid, card)| (id, cid, card))
            )
    }

    pub fn cur_ap(&self) -> i32 { self.cur_ap }

    pub fn max_ap(&self) -> i32 {
        self.parts.map().values()
            .map(|part| part.ap)
            .sum()
    }

    pub fn fill_ap(&mut self) {
        self.cur_ap = self.max_ap();
    }

    pub fn spend_ap(&mut self, ap: i32) -> bool {
        if ap > self.cur_ap { return false; }
        self.cur_ap -= ap;
        true
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Kind {
    Player(Player),
    NPC(NPC),
}

#[derive(Debug, Clone, Serialize)]
pub struct Player { }

#[derive(Debug, Clone, Serialize)]
pub struct NPC {
    pub move_range: i32,
    pub attack_range: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Part {
    pub cards: IdMap<Card>,
    pub ap: i32,
    /*
    power: i32,
    max_hp: i32,
    cur_hp: i32,
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