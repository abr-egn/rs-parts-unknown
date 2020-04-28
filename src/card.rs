use hex::Hex;
use serde::{Deserialize, Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    creature::{Creature},
    event::{Event},
    id_map::Id,
    part::{Part, PartTag},
    serde_empty,
    world::World,
    some_or,
};

#[derive(Clone, Serialize)]
pub struct Card {
    pub name: String,
    pub ap_cost: i32,
    // Contract: the world will not change between start_play and Behavior methods.
    #[serde(skip)]
    pub start_play: fn(&World, &Id<Creature>, &Id<Part>) -> Box<dyn Behavior>,
}

#[derive(Clone)]
pub struct InPlay {
    pub creature_id: Id<Creature>,
    pub part_id: Id<Part>,
    pub card_id: Id<Card>,
    pub behavior: Box<dyn Behavior>,
    pub ap_cost: i32,
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Card")
            .field("name", &self.name)
            .field("ap_cost", &self.ap_cost)
            .field("start_play", &(self.start_play as usize))
            .finish()
    }
}

// TASK: power scaling
pub trait Behavior: BehaviorClone {
    fn range(&self, world: &World) -> Vec<Hex>;
    // TASK: allow for multiple targets
    fn target_spec(&self) -> TargetSpec;
    fn target_check(&self, world: &World, target: &Target) -> bool;
    fn simulate(&self, world: &World, target: &Target) -> Vec<Event> {
        let mut tmp = world.clone();
        tmp.tracer = None;
        self.apply(&mut tmp, target)
    }
    fn apply(&self, world: &mut World, target: &Target) -> Vec<Event>;
}

impl dyn Behavior {
    pub fn target_valid(&self, world: &World, target: &Target) -> bool {
        if !self.target_spec().matches(world, target) { return false; }
        let range = self.range(world);
        if !range.is_empty() {
            let pos = some_or!(target.hex(world), return false);
            if !range.contains(&pos) { return false; }
        }
        self.target_check(world, target)
    }
}

// TASK: multiple targets
// May be better to just go to Parts { tags, count } rather than full
// generic Multi(Vec<TargetSpec>)
#[derive(Debug, Serialize, TsData)]
pub enum TargetSpec {
    #[serde(with = "serde_empty")]
    None,
    Part { on_player: bool, tags: Vec<Vec<PartTag>> /* Or<<X and Y>, <Q and R>> */ },
    #[serde(with = "serde_empty")]
    Creature,
}

impl TargetSpec {
    pub fn matches(&self, world: &World, target: &Target) -> bool {
        match (self, target) {
            (TargetSpec::None, Target::None) => true,
            (TargetSpec::Part { on_player, tags }, Target::Part { creature_id, part_id }) => {
                if *on_player != (*creature_id == world.player_id()) { return false; }
                let creature = some_or!(world.creatures().get(*creature_id), return false);
                let part = some_or!(creature.parts.get(*part_id), return false);
                for group in tags {
                    if group.iter().all(|tag| part.tags().contains(tag)) {
                        return true;
                    }
                }
                false
            }
            (TargetSpec::Creature, Target::Creature { id }) => *id != world.player_id(),
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, TsData)]
pub enum Target {
    #[serde(with = "serde_empty")]
    None,
    Part { creature_id: Id<Creature>, part_id: Id<Part> },
    Creature { id: Id<Creature> },
}

impl Target {
    pub fn hex(&self, world: &World) -> Option<Hex> {
        match self {
            Target::None => None,
            Target::Part { creature_id: id, .. } |
            Target::Creature { id } => {
                world.map().creatures().get(id).cloned()
            }
        }
    }
    pub fn part(&self) -> Option<(Id<Creature>, Id<Part>)> {
        match self {
            Target::Part { creature_id, part_id } => Some((*creature_id, *part_id)),
            _ => None
        }
    }
    pub fn creature(&self) -> Option<Id<Creature>> {
        match self {
            Target::Creature { id } => Some(*id),
            _ => None
        }
    }
}

pub trait BehaviorClone {
    fn clone_box(&self) -> Box<dyn Behavior>;
}

impl<T: 'static + Behavior + Clone> BehaviorClone for T {
    fn clone_box(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Behavior> {
    fn clone(&self) -> Self { self.clone_box() }
}