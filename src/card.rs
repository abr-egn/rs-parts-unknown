use hex::Hex;
use serde::{Deserialize, Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    creature::{Creature, Part, PartTag},
    event::{Event},
    id_map::Id,
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

// TODO: use this
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

// TODO: power scaling
pub trait Behavior: BehaviorClone {
    fn range(&self, _world: &World) -> Vec<Hex>;
    // TODO: allow for multiple targets
    fn target_spec(&self) -> TargetSpec;
    fn target_valid(&self, world: &World, target: &Target) -> bool {
        self.target_spec().matches(world, target)
    }
    fn simulate(&self, world: &World, target: &Target) -> Vec<Event> {
        let mut tmp = world.clone();
        tmp.tracer = None;
        self.apply(&mut tmp, target)
    }
    fn apply(&self, world: &mut World, target: &Target) -> Vec<Event>;
}

// TODO: multiple targets
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