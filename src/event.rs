use hex::Hex;
use serde::{Deserialize, Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::{
    creature::{Creature, CreatureAction, CreatureEvent},
    error::Error,
    id_map::Id,
    serde_empty,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TsData)]
pub enum Action {
    #[serde(with = "serde_empty")]
    Nothing,
    MoveCreature { id: Id<Creature>, to: Hex },
    //HitCreature { id: Id<Creature>, damage: i32 },
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

pub trait Trigger: TriggerClone + std::fmt::Debug + Send {
    fn name(&self) -> &'static str;
    fn applies(&self, action: &Action) -> bool;
    fn apply(&mut self, action: &Action, event: &Event) -> Vec<Action>;
}

pub type TriggerId = Id<Box<dyn Trigger>>;

pub trait TriggerClone {
    fn clone_box(&self) -> Box<dyn Trigger>;
}

impl<T> TriggerClone for T
where
    T: 'static + Trigger + Clone,
{
    fn clone_box(&self) -> Box<dyn Trigger> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Trigger> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}