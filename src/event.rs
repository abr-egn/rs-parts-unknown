use hex::Hex;
use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::{
    creature::{Creature, CreatureAction, CreatureEvent},
    error::Error,
    id_map::Id,
    serde_empty,
};

#[derive(Debug, Clone, Serialize, TsData)]
pub enum Action {
    #[serde(with = "serde_empty")]
    Nothing,
    MoveCreature { id: Id<Creature>, to: Hex },
    ToCreature { id: Id<Creature>, action: CreatureAction },
}

#[derive(Debug, Clone, Serialize, TsData)]
pub enum Event {
    #[serde(with = "serde_empty")]
    Nothing,
    Failed { action: Action, reason: String },
    CreatureMoved { id: Id<Creature>, from: Hex, to: Hex, },
    OnCreature { id: Id<Creature>, event: CreatureEvent },
}

impl Event {
    pub fn failed(err: Error) -> Event {
        Event::Failed { action: Action::Nothing, reason: format!("{:?}", err) }
    }
    pub fn is_failure(events: &[Event]) -> bool {
        match events {
            [Event::Failed { .. }, ..] => true,
            _ => false,
        }
    }
}