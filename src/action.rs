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

pub struct Meta<T> {
    pub source: Path,
    pub target: Path,
    pub tags: HashSet<Tag>,
    pub data: T,
}

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

pub struct Pending {
    pub action: Action,
    pub event: Event,
}