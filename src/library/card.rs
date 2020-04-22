use std::collections::HashSet;

use hex::Hex;

use crate::{
    card::{self, Card, Target, TargetSpec},
    creature::{Creature, CreatureAction, Part, PartAction, PartTag},
    event::{Action, Event},
    id_map::Id,
    world::World,
    some_or,
};

#[derive(Debug, Clone)]
pub struct Walk {
    creature_id: Id<Creature>,
}

impl card::Behavior for Walk {
    fn target_spec(&self) -> TargetSpec { TargetSpec::None }
    fn apply(&self, world: &mut World, _target: &Target) -> Vec<Event> {
        world.execute(&Action::ToCreature {
            id: self.creature_id,
            action: CreatureAction::GainMP { mp: 1 },
        })
    }
}

impl Walk {
    pub fn card() -> Card {
        Card {
            name: "Walk".into(),
            ap_cost: 1,
            start_play: |_, source, _| Walk::behavior(source),
        }
    }
    fn behavior(source: &Id<Creature>) -> Box<dyn card::Behavior> {
        Box::new(Walk { creature_id: *source })
    }
}

struct HitPart {
    damage: i32,
    tags: Vec<Vec<PartTag>>,
    melee: bool,
}

impl HitPart {
    fn behavior(self, world: &World, source: &Id<Creature>, part: &Id<Part>) -> Box<dyn card::Behavior> {
        let position = world.map().creatures().get(source).unwrap().clone();
        let range = if self.melee {
            position.neighbors().collect()
        } else {
            world.map().los_from(position)
        };
        let creature = world.creatures().get(*source).unwrap();
        let damage = creature.scale_damage_from(self.damage, Some(*part));
        Box::new(HitPartBehavior { damage, tags: self.tags, source: *source, range })
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
    fn apply(&self, world: &mut World, target: &Target) -> Vec<Event> {
        let (creature_id, part_id) = match target {
            Target::Part { creature_id, part_id } => (*creature_id, *part_id),
            _ => return vec![],
        };
        let source_creature = world.creatures().get(self.source).unwrap();
        let source_mp = source_creature.cur_mp;
        let mut out = vec![];
        out.extend(world.execute(&Action::ToCreature {
            id: self.source,
            action: CreatureAction::SpendMP { mp: source_mp },
        }));
        out.extend(world.execute(&Action::ToCreature {
            id: creature_id,
            action: CreatureAction::ToPart {
                id: part_id,
                action: PartAction::Hit { damage: self.damage }
            },
        }));
        out
    }
}

pub fn throw_debris() -> Card {
    Card {
        name: "Throw Debris".into(),
        ap_cost: 1,
        start_play: |world, source, part| HitPart { damage: 5, tags: vec![vec![]], melee: false }.behavior(world, source, part),
    }
}

pub fn punch() -> Card {
    Card {
        name: "Punch".into(),
        ap_cost: 1,
        start_play: |world, source, part| HitPart { damage: 10, tags: vec![vec![]], melee: true }.behavior(world, source, part),
    }
}

/* 10 cards:
arms:
    [+] 2 melee heavy attack
    [+] 2 ranged light attack
    [ ] 2 defense
legs:
    [ ] 2 open change
torso:
    [ ] 1 heal
head:
    [ ] 1 utility (draw, ?)
*/

//pub fn jab