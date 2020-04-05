use std::convert::TryInto;
use hex::Hex;
use crate::{
    creature::Creature,
    error::Error,
    event::{self, Action, Event, Meta},
    id_map::Id,
    world::World,
};

#[derive(Clone)]
pub struct Card {
    name: String,
    ap_cost: i32,
    start_play: fn(&World, &Id<Creature>) -> Box<dyn Behavior>,
}

impl Card {
    pub fn name(&self) -> &str { &self.name }
    pub fn ap_cost(&self) -> i32 { self.ap_cost }
    pub fn start_play(&self, world: &World, source: &Id<Creature>) -> Box<dyn Behavior> {
        (self.start_play)(world, source)
    }
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Card")
            .field("name", &self.name)
            .field("ap_cost", &self.ap_cost)
            .finish()
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
        let range: usize = self.range.try_into().unwrap();
        match world.map().path_to(self.start, cursor) {
            Ok(path) => path.len() <= range + 1,
            _ => false,
        }
    }
    fn apply(&self, world: &mut World, target: Hex) -> Vec<Meta<Event>> {
        let path = match world.map().path_to(self.start, target) {
            Ok(p) => p,
            Err(e) => return vec![event::failure(e)],
        };
        let mut out = vec![];
        for (from, to) in path.iter().zip(path.iter().skip(1)) {
            let actual = match world.map().creatures().get(&self.creature_id) {
                Some(h) => h,
                None => {
                    out.push(event::failure(Error::NoSuchCreature));
                    return out;
                }
            };
            if actual != from && actual.distance_to(*to) > 1 {
                out.push(event::failure(Error::Obstructed));
                return out;
            }
            out.append(&mut world.execute(&Meta::new(
                Action::MoveCreature { id: self.creature_id, to: *to }
            )));
        }
        out
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