use std::{
    collections::HashSet,
};

use hex::Hex;

use crate::{
    id_map::{Id},
    card::Card,
    creature::Creature,
    part::Part,
    trigger::{Trigger, TriggerId},
};

#[derive(Debug, Clone)]
pub struct Meta<T> {
    pub source: Path,
    pub target: Path,
    pub tags: HashSet<Tag>,
    pub data: T,
}

impl<T> Meta<T> {
    pub fn carry<D>(&self, data: D) -> Meta<D> {
        Meta {
            source: self.source.clone(),
            target: self.target.clone(),
            tags: self.tags.clone(),
            data,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Path {
    Global,
    Creature { cid: Id<Creature> },
    Part { cid: Id<Creature>, pid: Id<Part> },
    Card { cid: Id<Creature>, pid: Id<Part>, card: Id<Card> },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Tag {
    Attack,
    Normal,
}

#[derive(Debug, Clone)]
pub enum Action {
    // Global
    AddTrigger { trigger: Box<dyn Trigger> },
    RemoveTrigger { id: TriggerId },

    // Creature
    Move { to: Hex },
    GainAP { ap: i32 },
    SpendAP { ap: i32 },
    GainMP { mp: i32 },
    SpendMP { mp: i32 },
    NewHand,

    // Card
    Discard,
}

#[derive(Debug, Clone)]
pub enum Event {
    // Global
    TriggerAdded { id: TriggerId },
    TriggerRemoved { id: TriggerId },

    // Creature
    CreatureMoved { id: Id<Creature>, from: Hex, to: Hex, },
}