use std::collections::HashSet;

use hex::Hex;

use crate::{
    card::{self, Card},
    creature::{Creature, CreatureAction},
    event::{Action, Event},
    id_map::Id,
    map::Tile,
    world::World,
};

#[derive(Debug, Clone)]
pub struct Walk {
    range: i32,
    creature_id: Id<Creature>,
    start: Hex,
}

impl card::Behavior for Walk {
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
    pub fn card() -> Card {
        Card {
            name: "Walk".into(),
            ap_cost: 1,
            start_play: |world, source| Walk::behavior(world, source, 2),
        }
    }
    fn behavior(world: &World, source: &Id<Creature>, range: i32) -> Box<dyn card::Behavior> {
        let start = world.map().creatures().get(source).unwrap().clone();
        Box::new(Walk { range, creature_id: *source, start })
    }
}

#[derive(Debug, Clone)]
pub struct Shoot {
    source: Id<Creature>,
    los: HashSet<Hex>,
    damage: i32,
}

impl Shoot {
    pub fn behavior(world: &World, source: &Id<Creature>, damage: i32) -> Box<dyn card::Behavior> {
        let pos = world.map().creatures().get(source).unwrap().clone();
        let los = world.map().los_from(pos);
        Box::new(Shoot { source: *source, los, damage })
    }
    pub fn card() -> Card {
        Card {
            name: "Shoot".into(),
            ap_cost: 1,
            start_play: |world, source| Shoot::behavior(world, source, 1),
        }
    }
}

impl card::Behavior for Shoot {
    fn range(&self, _world: &World) -> Vec<Hex> {
        self.los.iter().cloned().collect()
    }
    fn highlight(&self, world: &World, _cursor: Hex) -> Vec<Hex> {
        self.range(world).into_iter()
            .filter(|hex| self.target_valid(world, *hex))
            .collect()
    }
    fn target_valid(&self, world: &World, cursor: Hex) -> bool {
        if !self.los.contains(&cursor) { return false; }
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
        let creature = world.creatures().get(target_id).unwrap();
        let action = Action::ToCreature {
            id: target_id,
            action: creature.hit_action(self.damage),
        };
        world.execute(&action)
    }
}