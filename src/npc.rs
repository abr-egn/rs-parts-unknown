use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    creature::{Creature},
    error::{Result},
    event::{Event},
    id_map::Id,
    world::World,
};

#[derive(Debug, Clone)]
pub struct NPC {
    pub next_motion: Option<Motion>,
    pub next_action: Option<Action>,
    pub behavior: Box<dyn Behavior>,
}

impl NPC {
    pub fn new(behavior: Box<dyn Behavior>) -> Self {
        NPC { next_motion: None, next_action: None, behavior }
    }
    pub fn update(&mut self, world: &World, id: Id<Creature>) {
        let (motion, action) = self.behavior.next(world, id);
        self.next_motion = motion;
        self.next_action = action;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Motion {
    ToMelee,
    /* TODO: more npc motions
    ToRanged,
    ToCover,
    */
}

#[derive(Clone)]
pub struct Action {
    pub kind: ActionKind,
    pub run: fn(&mut World, Id<Creature>) -> Result<Vec<Event>>,
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Action")
            .field("kind", &self.kind)
            .field("run", &(self.run as usize))
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TsData)]
pub enum ActionKind {
    Attack,
}

pub trait Behavior: BehaviorClone + std::fmt::Debug + Send {
    fn next(&mut self, world: &World, id: Id<Creature>) -> (Option<Motion>, Option<Action>);
}

pub trait BehaviorClone {
    fn clone_box(&self) -> Box<dyn Behavior>;
}

impl<T> BehaviorClone for T
where T: 'static + Behavior + Clone,
{
    fn clone_box(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Behavior> {
    fn clone(&self) -> Self { self.clone_box() }
}