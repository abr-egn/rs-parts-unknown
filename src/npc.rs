use std::collections::HashSet;

use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    action::{Event, Meta, Path, Tag, action, event, to_creature},
    creature::{Creature},
    error::{Error, Result},
    id_map::Id,
    part::{Part, PartTag},
    serde_empty,
    world::World,
    world_ext::WorldExt,
};

#[derive(Debug, Clone)]
pub struct NPC {
    pub intent: Intent,
    pub behavior: Box<dyn Behavior>,
}

impl NPC {
    pub fn update(&mut self, world: &World, id: Id<Creature>) {
        let intents = self.behavior.intent(world, id);
        for intent in intents {
            if intent.check(world, id).is_ok() {
                self.intent = intent;
                return;
            }
        }
        // Fallthrough
        self.intent = Intent {
            name: "Stunned".into(),
            from: None,
            cost: 0,
            kind: IntentKind::Stunned,
        }
    }
}

#[derive(Debug, Clone, Serialize, TsData)]
pub struct Intent {
    pub name: String,
    pub from: Option<Id<Part>>,
    pub cost: i32,
    pub kind: IntentKind,
}

impl Intent {
    pub fn move_(&self, world: &mut World, source: Id<Creature>) -> Result<Vec<Event>> {
        self.kind.move_(world, source)
    }

    pub fn act(&self, world: &mut World, source: Id<Creature>) -> Result<Vec<Event>> {
        self.check(world, source)?;
        self.kind.check(world, source)?;

        // Execute cost
        let mut act = to_creature(source, action::SpendAP { ap: 1 });
        act.tags.insert(Tag::Normal);
        let mut events = world.execute(&act);
        if Event::is_failure(&events) { return Ok(events); }

        // Execute action
        events.extend(self.kind.act(world, source, self.from));

        Ok(events)
    }

    fn check(&self, world: &World, source: Id<Creature>) -> Result<()> {
        // Check cost
        let creature = world.creatures().get(source).ok_or(Error::NoSuchCreature)?;
        if creature.cur_ap < self.cost {
            return Err(Error::NotEnough("AP".into()));
        }
        // Check part
        if let Some(part_id) = self.from {
            let part = creature.parts.get(part_id).ok_or(Error::NoSuchPart)?;
            if part.tags().contains(&PartTag::Broken) {
                return Err(Error::BrokenPart);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, TsData)]
pub enum IntentKind {
    Attack { damage: i32, range: Range },
    #[serde(with = "serde_empty")]
    Stunned,
}

impl IntentKind {
    fn check(&self, world: &World, source: Id<Creature>) -> Result<()> {
        match self {
            IntentKind::Attack { range, .. } => {
                let creature_pos = world.map().creatures().get(&source).ok_or(Error::OutOfBounds)?;
                let player_pos = world.map().creatures().get(&world.player_id()).ok_or(Error::OutOfBounds)?;
                let dist = creature_pos.distance_to(*player_pos);
                match range {
                    Range::Melee => if dist != 1 { return Err(Error::OutOfRange); }
                }
                Ok(())
            }
            IntentKind::Stunned => Ok(()),
        }
    }

    fn move_(&self, world: &mut World, source: Id<Creature>) -> Result<Vec<Event>> {
        match self {
            IntentKind::Attack { range: Range::Melee, .. } => move_to_melee(world, source),
            IntentKind::Stunned => Ok(vec![]),
        }
    }

    fn act(&self, world: &mut World, source: Id<Creature>, _part: Option<Id<Part>>) -> Vec<Event> {
        match self {
            IntentKind::Attack { damage, .. } => {
                let player_id = world.player_id();
                let pid = {
                    let player = world.creatures().get(player_id).unwrap();
                    let mut open: Vec<_> = player.open_parts().collect();
                    if open.is_empty() { return vec![]; }
                    open.sort_by(|(_, a), (_, b)| a.cur_hp.cmp(&b.cur_hp));
                    let (pid, _) = open.first().unwrap();
                    *pid
                };
                world.execute(&Meta {
                    source: Path::Creature { cid: source },
                    target: Path::Part { cid: player_id, pid },
                    tags: HashSet::new(), // TODO: attack
                    data: action::Hit { damage: *damage },
                })
            }
            IntentKind::Stunned => vec![to_creature(source, event::FloatText { text: "Stunned!".into() })]
        }
    }
}

#[derive(Debug, Clone, Serialize, TsData)]
pub enum Range {
    Melee,
    // TASK: Ranged
}

pub trait Behavior: BehaviorClone + std::fmt::Debug + Send {
    fn intent(&mut self, world: &World, id: Id<Creature>) -> Vec<Intent>;
}

pub trait BehaviorClone {
    fn clone_box(&self) -> Box<dyn Behavior>;
}

impl<T> BehaviorClone for T
where T: 'static + Behavior + Clone,
{
    fn clone_box(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Behavior> {
    fn clone(&self) -> Self { self.clone_box() }
}

fn move_to_melee(world: &mut World, id: Id<Creature>) -> Result<Vec<Event>> {
    let map = world.map();
    let player_hex = map.creatures().get(&world.player_id())
        .ok_or(Error::NoSuchCreature)?;
    let from = map.creatures().get(&id)
        .ok_or(Error::NoSuchCreature)?;
    if from.distance_to(*player_hex) <= 1 { return Ok(vec![]); }
    let mut near: Vec<_> = player_hex.neighbors()
        .filter(|h| map.tiles().get(h).map_or(false, |t| t.is_open()))
        .collect();
    if near.is_empty() { return Err(Error::Obstructed); }
    near.sort_by(|a, b| from.distance_to(*a).cmp(&from.distance_to(*b)));
    Ok(world.move_creature(id, near[0]))
}