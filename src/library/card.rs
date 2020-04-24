use std::collections::HashSet;

use hex::Hex;
use rand::prelude::*;

use crate::{
    card::{self, Card, Target, TargetSpec},
    creature::{Creature, CreatureAction, Part, PartAction, PartTag, TagModId},
    event::{Action, Event},
    id_map::Id,
    trigger::{Trigger, TriggerKind},
    world::World,
    some_or,
};

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
    fn target_spec(&self) -> TargetSpec { TargetSpec::Part { on_player: false, tags: self.tags.clone() } }
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
        if !part.tags().contains(&PartTag::Open) { return false; }
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

#[derive(Debug, Clone)]
struct ExpireTagMod {
    creature: Id<Creature>,
    part: Id<Part>,
    mod_: TagModId,
}

impl Trigger for ExpireTagMod {
    fn name(&self) -> &'static str { "Expire Tag Mod" }
    fn kind(&self) -> TriggerKind { TriggerKind::Expire }
    fn apply(&mut self, _event: &Event) -> Vec<Action> { vec![] }
}

pub fn guard() -> Card {
    Card {
        name: "Guard".into(),
        ap_cost: 1,
        start_play: |_, source, part| Box::new(Guard { source_creature: *source, source_part: *part })
    }
}

// TODO: this should only last 1 turn
#[derive(Debug, Clone)]
struct Guard {
    source_creature: Id<Creature>,
    source_part: Id<Part>,
}

impl card::Behavior for Guard {
    fn range(&self, _world: &World) -> Vec<Hex> { vec![] }
    fn target_spec(&self) -> TargetSpec {
        TargetSpec::Part { on_player: true, tags: vec![vec![PartTag::Open]] }
    }
    fn target_valid(&self, world: &World, target: &Target) -> bool {
        if !self.target_spec().matches(world, target) { return false; }
        let part_id = match target {
            Target::Part { part_id, .. } => *part_id,
            _ => panic!("invalid target"),
        };
        part_id != self.source_part
    }
    fn apply(&self, world: &mut World, target: &Target) -> Vec<Event> {
        let (target_id, part_id) = match target {
            Target::Part { creature_id, part_id } => (*creature_id, *part_id),
            _ => panic!("invalid target"),
        };
        let mut out = vec![];
        out.extend(world.execute(&Action::ToCreature {
            id: self.source_creature,
            action: CreatureAction::ToPart {
                id: self.source_part,
                action: PartAction::SetTags { tags: vec![PartTag::Open] },
            }
        }));
        if Event::is_failure(&out) {
            return out;
        }
        out.extend(world.execute(&Action::ToCreature {
            id: target_id,
            action: CreatureAction::ToPart {
                id: part_id,
                action: PartAction::ClearTags { tags: vec![PartTag::Open] },
            }
        }));
        out
    }
}

pub fn stagger() -> Card {
    Card {
        name: "Stagger".into(),
        ap_cost: 1,
        start_play: |_, _, _| Box::new(Stagger)
    }
}

#[derive(Debug, Clone)]
struct Stagger;

impl card::Behavior for Stagger {
    fn range(&self, world: &World) -> Vec<Hex> {
        let pos = world.map().creatures().get(&world.player_id()).unwrap();
        pos.neighbors().collect()
    }
    fn target_spec(&self) -> TargetSpec {
        TargetSpec::Creature
    }
    fn target_valid(&self, world: &World, target: &Target) -> bool {
        if !self.target_spec().matches(world, target) { return false; }
        let (_, part_ids) = Stagger::target_parts(world, target);
        !part_ids.is_empty()
    }
    fn simulate(&self, _world: &World, _target: &Target) -> Vec<Event> {
        // TODO: some kind of cosmetic event
        vec![]
    }
    fn apply(&self, world: &mut World, target: &Target) -> Vec<Event> {
        let (target_id, part_ids) = Stagger::target_parts(world, target);
        if part_ids.is_empty() { return vec![]; }

        let ix = thread_rng().gen_range(0, part_ids.len());
        world.execute(&Action::ToCreature{
            id: target_id,
            action: CreatureAction::ToPart {
                id: part_ids[ix],
                action: PartAction::SetTags { tags: vec![PartTag::Open] }
            }
        })
    }
}

impl Stagger {
    fn target_parts(world: &World, target: &Target) -> (Id<Creature>, Vec<Id<Part>>) {
        let target_id = match target {
            Target::Creature { id } => *id,
            _ => panic!("invalid target"),
        };
        let creature = world.creatures().get(target_id).unwrap();
        let part_ids: Vec<_> = creature.parts.iter().filter_map(|(id, part)| {
            if part.tags().contains(&PartTag::Broken) { None }
            else { Some(*id) }
        }).collect();
        (target_id, part_ids)
    }
}

/* 10 cards:
arms:
    [+] 2 melee heavy attack
    [+] 2 ranged light attack
    [+] 2 defense
legs:
    [ ] 2 stagger
torso:
    [ ] 1 heal
head:
    [ ] 1 utility (draw, ?)
*/

//pub fn jab