use std::{
    collections::HashSet,
};

use hex::Hex;
use serde::{Deserialize, Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    id_map::{Id},
    card::Card,
    creature::Creature,
    error::Error,
    part::{Part, PartTag, TagMod, TagModId},
    serde_empty,
    status::{Status, StatusId},
    world::World,
};

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize, TsData, PartialEq, Eq, Hash)]
pub enum Path {
    #[serde(with="serde_empty")]
    Global,  // TODO: rename to World
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

    pub fn hex(&self, world: &World) -> Option<Hex> {
        self.creature().and_then(|cid| world.map().creatures().get(&cid).cloned())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, TsData)]
pub enum Tag {
    Attack,
    Normal,  // TODO: rename to NoRender
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

    // Part
    Hit { damage: i32 },
    Heal { hp: i32 },
    SetTags { tags: Vec<PartTag> },
    ClearTags { tags: Vec<PartTag> },
    AddTagMod { m: TagMod },
    ClearTagMod { id: TagModId, },
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

    // Entity
    StatusAdded { id: StatusId },
    StatusRemoved { id: StatusId },

    // Creature
    Moved { from: Hex, to: Hex, },
    ChangeAP { delta: i32 },
    ChangeMP { delta: i32 },
    #[serde(with = "serde_empty")]
    Died,
    #[serde(with = "serde_empty")]
    DeckRecycled,

    // Card
    #[serde(with = "serde_empty")]
    Discarded,
    #[serde(with = "serde_empty")]
    Drew,

    // Part
    ChangeHP { delta: i32 },
    TagsSet { tags: Vec<PartTag> },
    TagsCleared { tags: Vec<PartTag> },
    TagsModded { id: TagModId },
    TagsUnmodded { id: TagModId },

    // Cosmetic
    FloatText { text: String },
}

impl EventData {
    pub fn is_global(&self) -> bool {
        match self {
            EventData::PlayerTurnEnd => true,
            EventData::NpcTurnEnd => true,
            _ => false,
        }
    }
}

pub mod event {
    pub use super::EventData::*;
}

pub type Event = Meta<EventData>;

impl Event {
    pub fn is_global(&self) -> bool { self.data.is_global() }
}

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

pub fn to_creature<T>(cid: Id<Creature>, data: T) -> Meta<T> {
    Meta {
        source: Path::Global,
        target: Path::Creature { cid },
        tags: HashSet::new(),
        data,
    }
}