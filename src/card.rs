use hex::Hex;
use serde::Serialize;

use crate::{
    creature::{Creature},
    event::{Action, Event},
    id_map::Id,
    world::World,
};

#[derive(Clone, Serialize)]
pub struct Card {
    pub name: String,
    pub ap_cost: i32,
    // Contract: the world will not change between start_play and Behavior methods.
    #[serde(skip)]
    pub start_play: fn(&World, &Id<Creature>) -> Box<dyn Behavior>,
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
    fn highlight(&self, _world: &World, _cursor: Hex) -> Vec<Hex> { vec![] }
    // TODO: allow for multiple targets
    fn target_valid(&self, world: &World, cursor: Hex) -> bool;
    fn preview(&self, world: &World, target: Hex) -> Vec<Action>;
    fn apply(&self, world: &mut World, target: Hex) -> Vec<Event> {
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

/* TODO
#[derive(Debug, Clone)]
pub struct Source {
    creature: Id<Creature>,
    part: Id<Part>,
    location: Hex,
}
*/

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