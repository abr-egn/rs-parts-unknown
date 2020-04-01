use std::convert::TryInto;

use hex::Hex;
use wasm_bindgen::prelude::*;

use crate::creature::Creature;
use crate::event::{Event, Meta};
use crate::id_map::Id;
use crate::world::World;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Card {
    name: String,
    ap_cost: i32,
    start_play: fn(&World, &Id<Creature>) -> Box<dyn Behavior>,
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Card")
            .field("name", &self.name)
            .field("ap_cost", &self.ap_cost)
            .finish()
    }
}

impl Card {
    pub fn name(&self) -> &str { &self.name }
    pub fn start_play(&self, world: &World, source: &Id<Creature>) -> Box<dyn Behavior> {
        (self.start_play)(world, source)
    }
}

// TODO: power scaling
pub trait Behavior: BehaviorClone {
    fn highlight(&self, world: &World, cursor: Hex) -> Vec<Hex>;
    // TODO: allow for multiple targets
    fn target_valid(&self, world: &World, cursor: Hex) -> bool;
    fn apply(&self, world: &mut World, target: Hex) -> Vec<Meta<Event>>;
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

#[derive(Debug, Clone)]
pub struct Walk {
    range: i32,
    creature_id: Id<Creature>,
    start: Hex,
}

impl Behavior for Walk {
    fn highlight(&self, world: &World, _: Hex) -> Vec<Hex> {
        world.map().range_from(self.start, self.range).into_iter().collect()
    }
    fn target_valid(&self, world: &World, cursor: Hex) -> bool {
        match world.map().path_to(self.start, cursor) {
            Ok(path) => path.len() <= self.range.try_into().unwrap(),
            _ => false,
        }
    }
    fn apply(&self, world: &mut World, target: Hex) -> Vec<Meta<Event>> {
        world.move_player(target)
    }
}

impl Walk {
    pub fn behavior(world: &World, source: &Id<Creature>, range: i32) -> Box<dyn Behavior> {
        let start = world.map().creatures().get(source).unwrap().clone();
        Box::new(Walk { range, creature_id: *source, start })
    }
    pub fn card() -> Card {
        Card {
            name: "Walk".into(),
            ap_cost: 1,
            start_play: |world, source| Walk::behavior(world, source, 2),
        }
    }
}

mod wasm {
    use crate::display;

    use super::*;

    #[wasm_bindgen]
    impl Card {
        #[wasm_bindgen(js_name = clone)]
        pub fn js_clone(&self) -> Card { self.clone() }
        #[wasm_bindgen(getter = name)]
        pub fn js_name(&self) -> String { self.name.clone() }
        #[wasm_bindgen(getter = apCost)]
        pub fn ap_cost(&self) -> i32 { self.ap_cost }
        #[wasm_bindgen(js_name = startPlay)]
        pub fn js_start_play(&self, world: &World, source: u32) -> display::Behavior {
            let id: Id<Creature> = Id::synthesize(source);
            display::Behavior::new(self.start_play(world, &id))
        }
    }
}