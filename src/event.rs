use std::collections::HashSet;

use hex::Hex;

use crate::creature::Creature;
use crate::error::Error;
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
    pub fn tagged(data: T, tags: &[&str]) -> Self {
        Meta {
            data,
            tags: tags.iter().map(|&s| s.into()).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Nothing,
    MoveCreature { id: Id<Creature>, to: Hex },
    SpendAP { id: Id<Creature>, ap: i32 },
}

#[derive(Debug, Clone)]
pub enum Event {
    Nothing,
    Failed { action: Action, reason: String },
    CreatureMoved { id: Id<Creature>, from: Hex, to: Hex, },
    SpentAP { id: Id<Creature>, ap: i32 },
}

pub fn failure(err: Error) -> Meta<Event> {
    Meta::new(Event::Failed { action: Action::Nothing, reason: format!("{:?}", err) })
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