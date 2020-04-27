use std::collections::HashSet;

use hex::Hex;
use rand::prelude::*;

use crate::{
    card::{self, Card, Target, TargetSpec},
    creature::{Creature, CreatureAction},
    event::{Action, Event},
    id_map::Id,
    mod_stack::Mod,
    part::{Part, PartAction, PartTag, TagMod, TagModId, WorldExt},
    trigger::{Trigger, TriggerId, TriggerKind},
    world::World,
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
    fn target_check(&self, _world: &World, target: &Target) -> bool {
        let (creature_id, _) = target.part().unwrap();
        creature_id != self.source
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
        out.extend(world.execute(&Action::to_part(
            creature_id, part_id,
            PartAction::Hit { damage: self.damage }
        )));
        out
    }
}

pub fn throw_debris() -> Card {
    Card {
        name: "Throw Debris".into(),
        ap_cost: 1,
        start_play: |world, source, part| HitPart { damage: 5, tags: vec![vec![PartTag::Open]], melee: false }.behavior(world, source, part),
    }
}

pub fn punch() -> Card {
    Card {
        name: "Punch".into(),
        ap_cost: 1,
        start_play: |world, source, part| HitPart { damage: 10, tags: vec![vec![PartTag::Open]], melee: true }.behavior(world, source, part),
    }
}

#[derive(Clone)]
struct ExpireTagMod {
    creature: Id<Creature>,
    part: Id<Part>,
    mod_: TagModId,
    when: fn(&Event) -> bool,
}

impl std::fmt::Debug for ExpireTagMod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExpireTagMod")
            .field("creature", &self.creature)
            .field("part", &self.part)
            .field("mod", &self.mod_)
            .field("when", &(self.when as usize))
            .finish()
    }
}

impl ExpireTagMod {
    fn add(world: &mut World, creature: Id<Creature>, part: Id<Part>, m: TagMod, when: fn(&Event) -> bool) -> Vec<Event> {
        let mut out = vec![];
        let (open_id, open_evs) = world.add_mod(creature, part, m);
        out.extend(open_evs);
        out.extend(world.execute(&Action::AddTrigger {
            trigger: Box::new(ExpireTagMod {
                creature, part,
                mod_: open_id,
                when,
            })
        }));
        out
    }
}

impl Trigger for ExpireTagMod {
    fn name(&self) -> &'static str { "Expire Tag Mod" }
    fn kind(&self) -> TriggerKind { TriggerKind::Expire }
    fn apply(&mut self, this: TriggerId, event: &Event) -> Vec<Action> {
        if !(self.when)(event) { return vec![]; }
        vec![
            Action::to_part(
                self.creature, self.part,
                PartAction::ClearTagMod { id: self.mod_ }
            ),
            Action::RemoveTrigger { id: this },
        ]
    }
}

pub fn guard() -> Card {
    Card {
        name: "Guard".into(),
        ap_cost: 1,
        start_play: |_, source, part| Box::new(Guard { source_creature: *source, source_part: *part })
    }
}

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
    fn target_check(&self, _world: &World, target: &Target) -> bool {
        let (_, part_id) = target.part().unwrap();
        part_id != self.source_part
    }
    fn apply(&self, world: &mut World, target: &Target) -> Vec<Event> {
        let (target_id, part_id) = target.part().unwrap();
        let mut out = vec![];
        out.extend(ExpireTagMod::add(world,
            self.source_creature, self.source_part,
            Mod(|tags| { tags.insert(PartTag::Open); }),
            |ev| matches!(ev, Event::NpcTurnEnd)
        ));
        out.extend(ExpireTagMod::add(world,
            target_id, part_id,
            Mod(|tags| { tags.remove(&PartTag::Open); }),
            |ev| matches!(ev, Event::NpcTurnEnd)
        ));
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
    fn target_check(&self, world: &World, target: &Target) -> bool {
        let (_, part_ids) = Stagger::target_parts(world, target);
        !part_ids.is_empty()
    }
    fn simulate(&self, _world: &World, target: &Target) -> Vec<Event> {
        let creature_id = target.creature().unwrap();
        vec![Event::FloatText { on: creature_id, text: "Stagger!".into() }]
    }
    fn apply(&self, world: &mut World, target: &Target) -> Vec<Event> {
        let (target_id, part_ids) = Stagger::target_parts(world, target);
        if part_ids.is_empty() { return vec![]; }

        let ix = thread_rng().gen_range(0, part_ids.len());
        ExpireTagMod::add(world, target_id, part_ids[ix],
            Mod(|tags| { tags.insert(PartTag::Open); }),
            |ev| matches!(ev, Event::PlayerTurnEnd))
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

pub fn heal() -> Card {
    Card {
        name: "Heal".into(),
        ap_cost: 2,
        start_play: |_, _, _| Box::new(Heal { amount: 5 })
    }
}

#[derive(Debug, Clone)]
struct Heal {
    amount: i32,
}

impl card::Behavior for Heal {
    fn range(&self, _world: &World) -> Vec<Hex> { vec![] }
    fn target_spec(&self) -> TargetSpec {
        TargetSpec::Part { on_player: true, tags: vec![vec![PartTag::Flesh]] }
    }
    fn target_check(&self, _world: &World, _target: &Target) -> bool { true }
    fn apply(&self, world: &mut World, target: &Target) -> Vec<Event> {
        let (cid, pid) = target.part().unwrap();
        world.execute(&Action::to_part(cid, pid,
            PartAction::Heal { hp: self.amount }
        ))
    }
}

/* 10 cards:
arms:
    [+] 2 melee heavy attack
    [+] 2 ranged light attack
    [+] 2 defense
legs:
    [+] 2 stagger
torso:
    [+] 1 heal
head:
    [ ] 1 utility (draw, ?)
*/