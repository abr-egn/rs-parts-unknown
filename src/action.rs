use std::{
    collections::HashSet,
    marker::PhantomData,
};

use hex::Hex;

use crate::{
    id_map::{Id},
    creature::Creature,
    error::Result,
    part::Part,
    world::World,
};

pub struct Meta<T> {
    pub source: Path,
    pub target: Path,
    pub tags: HashSet<Tag>,
    pub data: T,
}

pub enum Path {
    Global,
    Creature { id: Id<Creature> },
    Part { cid: Id<Creature>, pid: Id<Part> },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Tag {
    Attack,
    Normal,
}

pub enum Action {
    MoveCreature { to: Hex },
}

impl Meta<Action> {
    pub fn resolve(&self, world: &World) -> Result<Event<Pending>> {
        unimplemented!()
    }
}

pub enum Event<T> {
    Never { p: PhantomData<T> },
}

pub struct Pending;
pub struct Done;