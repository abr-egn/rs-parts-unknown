use hex::Hex;
use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    creature::{Creature, CreatureAction, CreatureEvent},
    error::Error,
    id_map::Id,
    part::{Part, PartAction, PartEvent},
    serde_empty,
    trigger::{Trigger, TriggerId},
};

#[derive(Debug, Clone, Serialize, TsData)]
pub enum Action {
    #[serde(with = "serde_empty")]
    Nothing,
    MoveCreature { id: Id<Creature>, to: Hex },
    ToCreature { id: Id<Creature>, action: CreatureAction },
    AddTrigger {
        #[serde(skip)]
        trigger: Box<dyn Trigger>
    },
    RemoveTrigger { id: TriggerId },
}

impl Action {
    pub fn to_part(cid: Id<Creature>, pid: Id<Part>, action: PartAction) -> Self {
        Action::ToCreature {
            id: cid,
            action: CreatureAction::ToPart {
                id: pid,
                action,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, TsData)]
pub enum Event {
    #[serde(with = "serde_empty")]
    Nothing,
    Failed { action: Action, reason: String },
    CreatureMoved { id: Id<Creature>, from: Hex, to: Hex, },
    OnCreature { id: Id<Creature>, event: CreatureEvent },
    TriggerAdded { id: TriggerId },
    TriggerRemoved { id: TriggerId },
    #[serde(with = "serde_empty")]
    PlayerTurnEnd,
    #[serde(with = "serde_empty")]
    NpcTurnEnd,
    FloatText { on: Id<Creature>, text: String },
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
    pub fn on_part(&self) -> Option<(Id<Creature>, Id<Part>, &PartEvent)> {
        match self {
            Event::OnCreature { id, event: CreatureEvent::OnPart { id: pid, event } } => Some((*id, *pid, event)),
            _ => None,
        }
    }
}