use hex::Hex;
use serde::Serialize;
use crate::{
    creature::{Creature, CreatureAction},
    event::{Action, Event},
    id_map::Id,
    map::Tile,
    world::World,
};

#[derive(Clone, Serialize)]
pub struct Card {
    pub name: String,
    pub ap_cost: i32,
    #[serde(skip)]
    pub start_play: fn(&World, &Id<Creature>) -> Box<dyn Behavior>,
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

#[derive(Debug, Clone)]
pub struct Walk {
    range: i32,
    creature_id: Id<Creature>,
    start: Hex,
}

impl Behavior for Walk {
    fn highlight(&self, world: &World, _: Hex) -> Vec<Hex> {
        world.map().creatures().get(&self.creature_id).into_iter().cloned().collect()
    }
    fn target_valid(&self, world: &World, cursor: Hex) -> bool {
        Some(&cursor) == world.map().creatures().get(&self.creature_id)
    }
    fn preview(&self, _world: &World, _target: Hex) -> Vec<Action> {
        vec![Action::ToCreature {
            id: self.creature_id,
            action: CreatureAction::GainMP { mp: 1 },
        }]
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

#[derive(Debug, Clone)]
pub struct Shoot {
    source: Id<Creature>,
    source_pos: Hex,
    range: i32,
    damage: i32,
}

impl Shoot {
    pub fn behavior(world: &World, source: &Id<Creature>, range: i32, damage: i32) -> Box<dyn Behavior> {
        let pos = world.map().creatures().get(source).unwrap().clone();
        Box::new(Shoot { source: *source, source_pos: pos, range, damage })
    }
    pub fn card() -> Card {
        Card {
            name: "Shoot".into(),
            ap_cost: 1,
            start_play: |world, source| Shoot::behavior(world, source, 5, 1),
        }
    }
}

impl Behavior for Shoot {
    fn range(&self, world: &World) -> Vec<Hex> {
        // TODO: line of sight rather than movement range
        world.map().range_from(self.source_pos, self.range).into_iter().collect()
    }
    fn highlight(&self, world: &World, _cursor: Hex) -> Vec<Hex> {
        self.range(world).into_iter()
            .filter(|hex| self.target_valid(world, *hex))
            .collect()
    }
    fn target_valid(&self, world: &World, cursor: Hex) -> bool {
        if cursor.distance_to(self.source_pos) > self.range { return false; }
        match world.map().tiles().get(&cursor) {
            Some(Tile { creature: Some(id), ..}) if *id != self.source => true,
            _ => false,
        }
    }
    fn preview(&self, _world: &World, _target: Hex) -> Vec<Action> {
        vec![]
    }
    fn apply(&self, world: &mut World, target: Hex) -> Vec<Event> {
        let target_id = world.map().tiles().get(&target).unwrap().creature.unwrap();
        let creature = world.creatures().map().get(&target_id).unwrap();
        let action = Action::ToCreature {
            id: target_id,
            action: creature.hit_action(self.damage),
        };
        world.execute(&action)
    }
}