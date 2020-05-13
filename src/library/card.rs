use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

use enum_iterator::IntoEnumIterator;
use hex::Hex;
use rand::prelude::*;

use crate::{
    action::{Action, Event, Path, Tag, action, event, to_creature},
    card::{self, Card, TargetSpec},
    creature::{Creature},
    id_map::Id,
    mod_stack::Mod,
    part::{Part, PartTag, TagMod},
    status::{AlterOrder, Status, StatusDone, StatusKind},
    world::{Scope, World},
    world_ext::WorldExt,
};

struct HitPart {
    damage: i32,
    tags: Vec<Vec<PartTag>>,
    melee: bool,
}

impl HitPart {
    fn behavior(self, world: &World, source: &Path) -> Box<dyn card::Behavior> {
        let cid = source.creature().unwrap();
        let position = world.map().creatures().get(&cid).unwrap().clone();
        let range = if self.melee {
            position.neighbors().collect()
        } else {
            world.map().los_of(cid).unwrap()
        };
        Box::new(HitPartBehavior {
            damage: self.damage,
            tags: self.tags,
            range
        })
    }
}

#[derive(Debug, Clone)]
struct HitPartBehavior {
    damage: i32,
    tags: Vec<Vec<PartTag>>,
    range: HashSet<Hex>,
}

impl card::Behavior for HitPartBehavior {
    fn range(&self, _source: &Path, _world: &World) -> Vec<Hex> { self.range.iter().cloned().collect() }
    fn target_spec(&self) -> TargetSpec { TargetSpec::Part { on_player: false, tags: self.tags.clone() } }
    fn target_check(&self, _world: &World, source: &Path, target: &Path) -> bool {
        target.creature().unwrap() != source.creature().unwrap()
    }
    fn apply(&self, world: &mut World, source: Path, target: Path) -> Vec<Event> {
        let source_cid = source.creature().unwrap();
        let source_creature = world.creatures().get(source_cid).unwrap();
        let source_mp = source_creature.cur_mp;
        let mut out = vec![];
        out.extend(world.execute(&Action {
            source: Path::World,
            target: Path::Creature { cid: source_cid },
            tags: HashSet::from_iter(vec![Tag::NoRender]),
            data: action::SpendMP { mp: source_mp },
        }));
        out.extend(world.execute(&Action {
            source, target,
            tags: HashSet::from_iter(vec![Tag::Attack]),  // TODO: melee tag
            data: action::Hit { damage: self.damage },
        }));
        out
    }
}

fn no_ui(_world: &World, _source: &Path, _target: &Path) -> HashMap<String, String> {
    hash(vec![
        ("test", "value".into()),
    ])
}

fn scaled(prefix: &str, base: i32, new: Option<i32>) -> HashMap<String, String> {
    let (value, delta) = match new {
        None => (base, "unknown"),
        Some(value) => (value, if value > base {
            "increase"
        } else if value < base {
            "decrease"
        } else {
            "same"
        })
    };
    let mut out = HashMap::new();
    out.insert(format!("{}_value", prefix), format!("{}", value));
    out.insert(format!("{}_delta", prefix), delta.into());
    out
}

fn attack_ui(world: &World, source: &Path, target: &Path, base: i32) -> HashMap<String, String> {
    let target = if source.creature() == target.creature() {
        &Path::World
    } else {
        target
    };
    let (damage, action) = world.scale_damage(source, target, base, Scope::into_enum_iter());
    let mut out = scaled("damage", base, damage);
    // TODO: tags
    out
}

fn hash<'a, I: IntoIterator<Item=(&'a str, String)>>(i: I) -> HashMap<String, String> {
    i.into_iter().map(|(k, v)| (k.into(), v)).collect()
}

pub fn throw_debris() -> Card {
    static DAMAGE: i32 = 5;
    Card {
        name: "Throw Debris".into(),
        ap_cost: 1,
        start_play: |world, source| HitPart {
            damage: DAMAGE,
            tags: vec![vec![PartTag::Open]],
            melee: false,
        }.behavior(world, source),
        ui: |world, source, target| attack_ui(world, source, target, DAMAGE),
    }
}

pub fn punch() -> Card {
    static DAMAGE: i32 = 10;
    Card {
        name: "Punch".into(),
        ap_cost: 1,
        start_play: |world, source| HitPart {
            damage: DAMAGE,
            tags: vec![vec![PartTag::Open]],
            melee: true,
        }.behavior(world, source),
        ui: |world, source, target| attack_ui(world, source, target, DAMAGE),
    }
}

#[derive(Clone)]
struct Expire<When> {
    remove: Vec<Action>,
    when: When
}

impl<When: Fn(&Event) -> bool + Clone + 'static> Expire<When> {
    fn tag_mod(world: &mut World, target: &Path, m: TagMod, when: When) -> Vec<Event> {
        let mut out = world.execute(&Action {
            source: Path::World, target: target.clone(),
            tags: HashSet::new(),
            data: action::AddTagMod { m },
        });
        let mod_id = match &out as &[_] {
            [Event { data: event::TagsModded { id }, .. }, ..] => *id,
            _ => return out,
        };
        out.extend(world.execute(&Action {
            source: Path::World, target: target.clone(),
            tags: HashSet::new(),
            data: action::AddStatus {
                status: Box::new(Self {
                    remove: vec![Action {
                        source: Path::World,
                        target: target.clone(),
                        tags: HashSet::new(),
                        data: action::ClearTagMod { id: mod_id },
                    }],
                    when,
                })
            },
        }));
        out
    }
}

impl<When> std::fmt::Debug for Expire<When> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Expire")
            .finish()
    }
}

impl<When: Fn(&Event) -> bool + Clone + 'static> Status for Expire<When> {
    fn name(&self) -> &'static str { "Expire" }
    fn kind(&self) -> StatusKind { StatusKind::Hidden }
    fn trigger(&mut self, _on: &Path, event: &Event) -> (Vec<Action>, StatusDone) {
        if !(self.when)(event) { return (vec![], StatusDone::Continue); }
        (self.remove.clone(), StatusDone::Expire)
    }
}

pub fn guard() -> Card {
    Card {
        name: "Block".into(),
        ap_cost: 1,
        start_play: |_, _| Box::new(Guard),
        ui: no_ui,
    }
}

#[derive(Debug, Clone)]
struct Guard;

impl card::Behavior for Guard {
    fn range(&self, _source: &Path, _world: &World) -> Vec<Hex> { vec![] }
    fn target_spec(&self) -> TargetSpec {
        TargetSpec::Part { on_player: true, tags: vec![vec![PartTag::Open]] }
    }
    fn target_check(&self, _world: &World, source: &Path, target: &Path) -> bool {
        source.part() != target.part()
    }
    fn apply(&self, world: &mut World, source: Path, target: Path) -> Vec<Event> {
        let mut out = vec![];
        out.extend(Expire::tag_mod(world, &source,
            Mod(|tags| { tags.insert(PartTag::Open); }),
            |ev| matches!(ev, Event { data: event::NpcTurnEnd, .. })
        ));
        out.extend(Expire::tag_mod(world, &target,
            Mod(|tags| { tags.remove(&PartTag::Open); }),
            |ev| matches!(ev, Event { data: event::NpcTurnEnd, .. })
        ));
        out
    }
}

pub fn stagger() -> Card {
    Card {
        name: "Stagger".into(),
        ap_cost: 1,
        start_play: |_, _| Box::new(Stagger),
        ui: no_ui,
    }
}

#[derive(Debug, Clone)]
struct Stagger;

impl card::Behavior for Stagger {
    fn range(&self, _source: &Path, world: &World) -> Vec<Hex> {
        let pos = world.map().creatures().get(&world.player_id()).unwrap();
        pos.neighbors().collect()
    }
    fn target_spec(&self) -> TargetSpec {
        TargetSpec::Creature
    }
    fn target_check(&self, world: &World, _source: &Path, target: &Path) -> bool {
        !Stagger::target_parts(world, target.creature().unwrap()).is_empty()
    }
    fn preview(&self, _world: &World, _source: Path, target: Path) -> Vec<Event> {
        let creature_id = target.creature().unwrap();
        vec![to_creature(creature_id, event::FloatText { text: "Stagger!".into() })]
    }
    fn apply(&self, world: &mut World, _source: Path, target: Path) -> Vec<Event> {
        let cid = target.creature().unwrap();
        let part_ids = Stagger::target_parts(world, cid);
        if part_ids.is_empty() { return vec![]; }

        let ix = thread_rng().gen_range(0, part_ids.len());
        let (name, pid) = &part_ids[ix];
        let mut out = vec![];
        out.push(to_creature(cid, event::FloatText { text: format!("Exposed: {}", name) }));
        out.extend(Expire::tag_mod(world, &Path::Part { cid, pid: *pid },
            Mod(|tags| { tags.insert(PartTag::Open); }),
            |ev| matches!(ev, Event { data: event::PlayerTurnEnd, .. })));
        out
    }
}

impl Stagger {
    fn target_parts(world: &World, creature_id: Id<Creature>) -> Vec<(String, Id<Part>)> {
        let creature = world.creatures().get(creature_id).unwrap();
        let part_ids: Vec<_> = creature.parts.iter().filter_map(|(id, part)| {
            if part.tags().contains(&PartTag::Broken) { None }
            else { Some((part.name.clone(), *id)) }
        }).collect();
        part_ids
    }
}

pub fn heal() -> Card {
    static AMOUNT: i32 = 5;
    Card {
        name: "Regenerate".into(),
        ap_cost: 1,
        start_play: |_, _| Box::new(Heal { amount: AMOUNT }),
        ui: |_, _, _| { scaled("heal", AMOUNT, Some(AMOUNT)) }
    }
}

#[derive(Debug, Clone)]
struct Heal {
    amount: i32,
}

impl card::Behavior for Heal {
    fn range(&self, _source: &Path, _world: &World) -> Vec<Hex> { vec![] }
    fn target_spec(&self) -> TargetSpec {
        TargetSpec::Part { on_player: true, tags: vec![vec![PartTag::Flesh]] }
    }
    fn target_check(&self, world: &World, _source: &Path, target: &Path) -> bool {
        let (cid, pid) = target.part().unwrap();
        let creature = world.creatures().get(cid).unwrap();
        let part = creature.parts.get(pid).unwrap();
        part.cur_hp < part.max_hp
    }
    fn apply(&self, world: &mut World, source: Path, target: Path) -> Vec<Event> {
        world.execute(&Action {
            source, target,
            tags: HashSet::new(),
            data: action::Heal { hp: self.amount },
        })
    }
}

pub fn rage() -> Card {
    static AMOUNT: i32 = 7;
    Card {
        name: "Rage".into(),
        ap_cost: 1,
        start_play: |_, _| Box::new(Rage { amount: AMOUNT }),
        ui: |_, _, _| { scaled("added", AMOUNT, Some(AMOUNT)) }
    }
}

#[derive(Debug, Clone)]
struct Rage { amount: i32 }

impl card::Behavior for Rage {
    fn range(&self, _source: &Path, _world: &World) -> Vec<Hex> { vec![] }
    fn target_spec(&self) -> TargetSpec { TargetSpec::None }
    fn target_check(&self, _world: &World, _source: &Path, _target: &Path) -> bool { true }
    fn apply(&self, world: &mut World, source: Path, _target: Path) -> Vec<Event> {
        let cid = source.creature().unwrap();
        world.execute(&Action {
            source,
            target: Path::Creature { cid },
            tags: HashSet::new(),
            data: action::AddStatus { status: Box::new(self.clone()) },
        })
    }
}

impl Status for Rage {
    fn name(&self) -> &'static str { "Rage" }
    fn kind(&self) -> StatusKind { StatusKind::Buff }
    fn alter_order(&self) -> AlterOrder { AlterOrder::Add }
    fn alter(&mut self, on: &Path, action: &Action) -> Option<Action> {
        if on.creature() != action.source.creature() { return None; }
        let mut action = action.clone();
        match &mut action.data {
            action::Hit { damage } => {
                *damage = *damage + self.amount;
                Some(action)
            },
            _ => None
        }
    }
    fn trigger(&mut self, _on: &Path, event: &Event) -> (Vec<Action>, StatusDone) {
        match event.data {
            event::PlayerTurnEnd => (vec![], StatusDone::Expire),
            _ => (vec![], StatusDone::Continue),
        }
    }
}

pub fn debug_debuff() -> Card {
    Card {
        name: "Debug Debuff".into(),
        ap_cost: 0,
        start_play: |_, _| Box::new(DebugDebuff),
        ui: no_ui,
    }
}

#[derive(Debug, Clone)]
struct DebugDebuff;

impl card::Behavior for DebugDebuff {
    fn range(&self, source: &Path, world: &World) -> Vec<Hex> {
        let cid = source.creature().unwrap();
        world.map().los_of(cid).unwrap().into_iter().collect()
    }
    fn target_spec(&self) -> TargetSpec { TargetSpec::Creature }
    fn target_check(&self, _world: &World, _source: &Path, _target: &Path) -> bool { true }
    fn apply(&self, world: &mut World, source: Path, target: Path) -> Vec<Event> {
        world.execute(&Action {
            source, target,
            tags: HashSet::new(),
            data: action::AddStatus { status: Box::new(DebugDebuff) }
        })
    }
}

impl Status for DebugDebuff {
    fn name(&self) -> &'static str { "Debug" }
    fn kind(&self) -> StatusKind { StatusKind::Debuff }
    fn alter(&mut self, on: &Path, action: &Action) -> Option<Action> {
        if action.source.creature() != on.creature() { return None; }
        let mut action = action.clone();
        match &mut action.data {
            action::Hit { damage, .. } => {
                *damage = *damage - 1;
                Some(action)
            },
            _ => None,
        }
    }
}