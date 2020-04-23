use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    creature::{Creature, CreatureAction, Part, PartAction, PartTag},
    error::{Error, Result},
    event::{Action, Event},
    id_map::Id,
    world::World,
};

#[derive(Debug, Clone)]
pub struct NPC {
    pub next_motion: Option<Motion>,
    pub next_action: Option<Intent>,
    pub behavior: Box<dyn Behavior>,
}

impl NPC {
    pub fn new(behavior: Box<dyn Behavior>) -> Self {
        NPC { next_motion: None, next_action: None, behavior }
    }
    pub fn update(&mut self, world: &World, id: Id<Creature>) {
        let (motion, action) = self.behavior.next(world, id);
        self.next_motion = motion;
        self.next_action = action;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Motion {
    ToMelee,
    /* TODO: more npc motions
    ToRanged,
    ToCover,
    */
}

#[derive(Debug, Clone)]
pub struct Intent {
    pub from: Id<Part>, // TODO: redundancy?
    pub cost: i32,
    pub kind: IntentKind,
}

impl Intent {
    pub fn check_run(&self, world: &mut World, source: Id<Creature>) -> Result<Vec<Event>> {
        // Check cost
        let creature = world.creatures().get(source).ok_or(Error::NoSuchCreature)?;
        if creature.cur_ap < self.cost {
            return Err(Error::NotEnough);
        }
        // Check part
        let part = creature.parts.get(self.from).ok_or(Error::NoSuchPart)?;
        if part.tags.contains(&PartTag::Broken) {
            return Err(Error::NoSuchPart);
        }
        // Check kind
        self.kind.check(world, source)?;

        // Execute cost
        let mut events = world.execute(&Action::ToCreature {
            id: source,
            action: CreatureAction::SpendAP { ap: 1 },
        });
        if Event::is_failure(&events) { return Ok(events); }

        // Execute action
        events.extend(self.kind.run(world, source, self.from));

        Ok(events)
    }
}

#[derive(Debug, Clone, Serialize, TsData)]
pub enum IntentKind {
    Attack { base_damage: i32, range: Range }
}

impl IntentKind {
    fn check(&self, world: &World, source: Id<Creature>) -> Result<()> {
        match self {
            IntentKind::Attack { range, .. } => {
                let creature_pos = world.map().creatures().get(&source).ok_or(Error::OutOfBounds)?;
                let player_pos = world.map().creatures().get(&world.player_id()).ok_or(Error::OutOfBounds)?;
                let dist = creature_pos.distance_to(*player_pos);
                match range {
                    Range::Melee => if dist != 1 { return Err(Error::Obstructed); }
                }
                Ok(())
            }
        }
    }

    fn run(&self, world: &mut World, source: Id<Creature>, part: Id<Part>) -> Vec<Event> {
        match self {
            IntentKind::Attack { base_damage, .. } => {
                let player_id = world.player_id();
                let player = world.creatures().get(player_id).unwrap();
                let mut open: Vec<_> = player.open_parts().collect();
                if open.is_empty() { return vec![]; }
                open.sort_by(|(_, a), (_, b)| a.cur_hp.cmp(&b.cur_hp));
                let (pid, _) = open.first().unwrap();
                let creature = world.creatures().get(source).unwrap();
                let damage_from = creature.scale_damage_from(*base_damage, Some(part));
                let damage = player.scale_damage_to(damage_from, Some(*pid));
                let hit = Action::ToCreature {
                    id: player_id,
                    action: CreatureAction::ToPart {
                        id: *pid,
                        action: PartAction::Hit { damage },
                    }
                };
                world.execute(&hit)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, TsData)]
pub enum Range {
    Melee,
    // TODO: Ranged
}

pub trait Behavior: BehaviorClone + std::fmt::Debug + Send {
    fn next(&mut self, world: &World, id: Id<Creature>) -> (Option<Motion>, Option<Intent>);
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