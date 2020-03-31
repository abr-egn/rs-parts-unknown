use std::collections::HashSet;

use hex::Hex;
use serde::Serialize;

use crate::creature::Creature;
use crate::id_map::Id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meta<T> {
    pub data: T,
    pub tags: HashSet<String>,
}

impl<T> Meta<T> {
    pub fn new(data: T) -> Self {
        Meta {
            data,
            tags: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Action {
    MoveCreature { id: Id<Creature>, to: Hex },
}

#[derive(Debug, Clone, Serialize)]
pub enum Event {
    CreatureMoved { id: Id<Creature>, path: Vec<Hex>, },
    Failed { action: Action, reason: String },
}

pub trait Mod: ModClone + std::fmt::Debug + Send {
    fn name(&self) -> &'static str;
    fn apply(&mut self, action: &mut Meta<Action>);
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
    fn apply(&mut self, event: &Meta<Event>) -> Vec<Meta<Action>>;
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