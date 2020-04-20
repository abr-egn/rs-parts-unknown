use hex::Hex;
use serde::{Deserialize, Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    creature::{Creature, Part, PartTag},
    event::{Action, Event},
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
    pub start_play: fn(&World, &Id<Creature>) -> Box<dyn Behavior>,
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
    fn range(&self, _world: &World) -> Vec<Hex> { vec![] }
    // TODO: allow for multiple targets
    fn target_spec(&self) -> TargetSpec;
    fn target_valid(&self, world: &World, target: &Target) -> bool {
        self.target_spec().matches(world, target)
    }
    fn preview(&self, world: &World, target: &Target) -> Vec<Action>;
    fn apply(&self, world: &mut World, target: &Target) -> Vec<Event> {
        let mut out = vec![];
        for act in self.preview(world, target) {
            let events = world.execute(&act);
            let failed = Event::is_failure(&events);
            out.extend(events);
            if failed { break; }
        }
        out
    }
}

// TODO: multiple targets
// May be better to just go to Parts { tags, count } rather than full
// generic Multi(Vec<TargetSpec>)
#[derive(Debug, Serialize, TsData)]
pub enum TargetSpec {
    #[serde(with = "serde_empty")]
    None,
    Part { tags: Vec<Vec<PartTag>> /* Or<<X and Y>, <Q and R>> */ }
    // TODO: Creature,
}

impl TargetSpec {
    pub fn matches(&self, world: &World, target: &Target) -> bool {
        match self {
            TargetSpec::None => matches!(target, Target::None),
            TargetSpec::Part { tags } => {
                match target {
                    Target::Part { creature_id, part_id } => {
                        let creature = some_or!(world.creatures().get(*creature_id), return false);
                        let part = some_or!(creature.parts.get(*part_id), return false);
                        for group in tags {
                            if group.iter().all(|tag| part.tags.contains(tag)) {
                                return true;
                            }
                        }
                        return false;
                    }
                    _ => false,
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, TsData)]
pub enum Target {
    #[serde(with = "serde_empty")]
    None,
    Part { creature_id: Id<Creature>, part_id: Id<Part> }
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