use std::collections::HashSet;

use hex::Hex;

use crate::{
    card::{self, Card, Target, TargetSpec},
    creature::{Creature, CreatureAction, PartAction, PartTag},
    event::{Action},
    id_map::Id,
    world::World,
    some_or,
};

#[derive(Debug, Clone)]
pub struct Walk {
    range: i32,
    creature_id: Id<Creature>,
    start: Hex,
}

impl card::Behavior for Walk {
    fn target_spec(&self) -> TargetSpec { TargetSpec::None }
    fn preview(&self, _world: &World, _target: &Target) -> Vec<Action> {
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

struct HitPart {
    damage: i32,
    tags: Vec<Vec<PartTag>>,
    melee: bool,
}

impl HitPart {
    fn behavior(self, world: &World, source: &Id<Creature>) -> Box<dyn card::Behavior> {
        let position = world.map().creatures().get(source).unwrap().clone();
        let range = if self.melee {
            position.neighbors().collect()
        } else {
            world.map().los_from(position)
        };
        Box::new(HitPartBehavior { damage: self.damage, tags: self.tags, source: *source, range })
    }
}

#[derive(Debug, Clone)]
struct HitPartBehavior {
    damage: i32,
    tags: Vec<Vec<PartTag>>,
    source: Id<Creature>,
    range: HashSet<Hex>,
}

impl card::Behavior for HitPartBehavior {
    fn range(&self, _world: &World) -> Vec<Hex> { self.range.iter().cloned().collect() }
    fn target_spec(&self) -> TargetSpec { TargetSpec::Part { tags: self.tags.clone() } }
    fn target_valid(&self, world: &World, target: &Target) -> bool {
        if !self.target_spec().matches(world, target) { return false; }
        let (creature_id, part_id) = if let Target::Part { creature_id, part_id } = target {
            (*creature_id, *part_id)
        } else { return false; };
        if creature_id == self.source { return false; }
        let pos = some_or!(world.map().creatures().get(&creature_id), return false);
        if !self.range.contains(&pos) { return false; }
        let creature = some_or!(world.creatures().get(creature_id), return false);
        let part = some_or!(creature.parts.get(part_id), return false);
        if !part.tags.contains(&PartTag::Open) { return false; }
        true
    }
    fn preview(&self, _world: &World, target: &Target) -> Vec<Action> {
        let (creature_id, part_id) = match target {
            Target::Part { creature_id, part_id } => (*creature_id, *part_id),
            _ => return vec![],
        };
        vec![Action::ToCreature {
            id: creature_id,
            action: CreatureAction::ToPart {
                id: part_id,
                action: PartAction::Hit { damage: self.damage }
            },
        }]
    }
}

pub fn shoot() -> Card {
    Card {
        name: "Shoot".into(),
        ap_cost: 1,
        start_play: |world, source| HitPart { damage: 1, tags: vec![vec![]], melee: false }.behavior(world, source),
    }
}

/* 10 cards:
arms:
    2 melee heavy attack
    2 ranged light attack
    2 defense
legs:
    2 open change
torso:
    1 heal
head:
    1 utility (draw, ?)

*/

//pub fn jab