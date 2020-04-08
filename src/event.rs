use hex::Hex;
use serde::Serialize;
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::{
    creature::Creature,
    error::Error,
    id_map::Id,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TsData)]
pub enum Action {
    Nothing,
    MoveCreature { id: Id<Creature>, to: Hex },
    SpendAP { id: Id<Creature>, ap: i32 },
    SpendMP { id: Id<Creature>, mp: i32 },
    GainMP { id: Id<Creature>, mp: i32 },
}

#[derive(Debug, Clone, Serialize, TsData)]
pub enum Event {
    Nothing,
    Failed { action: Action, reason: String },
    CreatureMoved { id: Id<Creature>, from: Hex, to: Hex, },
    SpentAP { id: Id<Creature>, ap: i32 },
    ChangeMP { id: Id<Creature>, mp: i32 },
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

pub trait Mod: ModClone + std::fmt::Debug + Send {
    fn name(&self) -> &'static str;
    fn apply(&mut self, action: &mut Action);
}

pub trait ModClone {
    fn clone_box(&self) -> Box<dyn Mod>;
}

impl<T> ModClone for T
where
    T: 'static + Mod + Clone,
{
    fn clone_box(&self) -> Box<dyn Mod> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Mod> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait Trigger: TriggerClone + std::fmt::Debug + Send {
    fn name(&self) -> &'static str;
    fn apply(&mut self, event: &Event) -> Vec<Action>;
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