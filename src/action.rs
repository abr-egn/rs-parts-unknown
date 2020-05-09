use std::{
    collections::HashSet,
};

use hex::Hex;
use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    id_map::{Id},
    card::Card,
    creature::Creature,
    error::Error,
    part::Part,
    serde_empty,
    status::{Status, StatusId},
};

#[derive(Debug, Clone)]
pub struct Meta<T> {
    pub source: Path,
    pub target: Path,
    pub tags: HashSet<Tag>,
    pub data: T,
}

impl<T> Meta<T> {
    pub fn new(data: T) -> Self {
        Meta {
            source: Path::Global,
            target: Path::Global,
            tags: HashSet::new(),
            data,
        }
    }

    pub fn carry<D>(&self, data: D) -> Meta<D> {
        Meta {
            source: self.source.clone(),
            target: self.target.clone(),
            tags: self.tags.clone(),
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, TsData)]
pub enum Path {
    #[serde(with="serde_empty")]
    Global,
    Creature { cid: Id<Creature> },
    Part { cid: Id<Creature>, pid: Id<Part> },
    Card { cid: Id<Creature>, pid: Id<Part>, card: Id<Card> },
}

impl Path {
    pub fn creature(&self) -> Option<Id<Creature>> {
        match self {
            Path::Creature { cid }
            | Path::Part { cid, .. }
            | Path::Card { cid, .. }
            => Some(*cid),
            _ => None,
        }
    }

    pub fn part(&self) -> Option<(Id<Creature>, Id<Part>)> {
        match self {
            Path::Part { cid, pid } => Some((*cid, *pid)),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Tag {
    Attack,
    Normal,
}

#[derive(Debug, Clone)]
pub enum ActionData {
    // Special
    Nothing,
    Fail { description: String },

    // Entity
    AddStatus { status: Box<dyn Status> },
    RemoveStatus { id: StatusId },

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

pub mod action {
    pub use super::ActionData::*;
}

pub type Action = Meta<ActionData>;

#[derive(Debug, Clone, Serialize, TsData)]
pub enum EventData {
    // Special
    #[serde(with = "serde_empty")]
    Nothing,
    Failed { description: String },

    // Global
    #[serde(with = "serde_empty")]
    PlayerTurnEnd,
    #[serde(with = "serde_empty")]
    NpcTurnEnd,
    FloatText { text: String },

    // Entity
    StatusAdded { id: StatusId },
    StatusRemoved { id: StatusId },

    // Creature
    Moved { from: Hex, to: Hex, },
}

pub mod event {
    pub use super::EventData::*;
}

pub type Event = Meta<EventData>;

impl Event {
    pub fn failed(err: Error) -> Event {
        Meta::new(EventData::Failed { description: format!("{:?}", err) })
    }
    pub fn is_failure(events: &[Event]) -> bool {
        match events {
            [Meta { data: EventData::Failed { .. }, .. }, ..] => true,
            _ => false,
        }
    }
}