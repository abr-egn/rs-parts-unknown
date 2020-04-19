use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    creature::{Creature, CreatureAction},
    error::{Error, Result},
    event::{self, Event},
    id_map::Id,
    world::World,
};

#[derive(Debug, Clone)]
pub struct NPC {
    pub next_motion: Option<Motion>,
    pub next_action: Option<Action>,
    behavior: Box<dyn Behavior>,
}

impl NPC {
    pub fn update(&mut self, world: &World, id: Id<Creature>) {
        let (motion, action) = self.behavior.next(world, id);
        self.next_motion = motion;
        self.next_action = action;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Motion {
    ToMelee,
    // TODO: ToRanged { min: i32, max: i32 },
    // TODO: ToCover,
}

#[derive(Clone)]
pub struct Action {
    pub kind: ActionKind,
    pub run: fn(&mut World, Id<Creature>) -> Result<Vec<Event>>,
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Action")
            .field("kind", &self.kind)
            .field("run", &(self.run as usize))
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TsData)]
pub enum ActionKind {
    Attack,
}

trait Behavior: BehaviorClone + std::fmt::Debug + Send {
    fn next(&mut self, world: &World, id: Id<Creature>) -> (Option<Motion>, Option<Action>);
}

trait BehaviorClone {
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

#[derive(Debug, Clone)]
pub struct Monopod {
    kick_time: bool,
}

impl Behavior for Monopod {
    fn next(&mut self, _world: &World, _id: Id<Creature>) -> (Option<Motion>, Option<Action>) {
        let action = if self.kick_time {
            Some(Action {
                kind: ActionKind::Attack,
                run: Monopod::kick,
            })
        } else { None };
        self.kick_time = !self.kick_time;
        (Some(Motion::ToMelee), action)
    }
}

impl Monopod {
    pub fn npc() -> NPC {
        NPC {
            next_motion: None,
            next_action: None,
            behavior: Box::new(Monopod { kick_time: true }),
        }
    }

    fn kick(world: &mut World, id: Id<Creature>) -> Result<Vec<Event>> {
        let player_id = world.player_id();
        let player = world.creatures().get(player_id).unwrap();
        let hit = player.hit_action(1);
        CheckRun {
            world, id,
            part: "Fut",
            range: Range::Melee,
            cost: 1,
            actions: vec![event::Action::ToCreature {
                id: player_id,
                action: hit,
            }],
        }.go()
    }
}

struct CheckRun<'a> {
    world: &'a mut World,
    id: Id<Creature>,
    part: &'a str,
    range: Range,
    cost: i32,
    actions: Vec<event::Action>,
}

impl<'a> CheckRun<'a> {
    fn go(self) -> Result<Vec<Event>> {
        let world = self.world;
        // Check cost
        let creature = world.creatures().get(self.id).ok_or(Error::NoSuchCreature)?;
        if creature.cur_ap < self.cost {
            return Err(Error::NotEnough);
        }
        // Check part
        let part = self.part;
        if !creature.parts.values().any(|p| p.name == part && !p.dead) {
            return Err(Error::NoSuchPart);
        }
        // Check range
        let creature_pos = world.map().creatures().get(&self.id).ok_or(Error::OutOfBounds)?;
        let player_pos = world.map().creatures().get(&world.player_id()).ok_or(Error::OutOfBounds)?;
        let dist = creature_pos.distance_to(*player_pos);
        match self.range {
            Range::Melee => if dist != 1 { return Err(Error::Obstructed); }
        }

        // Execute cost
        let mut events = world.execute(&event::Action::ToCreature {
            id: self.id,
            action: CreatureAction::SpendAP { ap: 1 },
        });
        if Event::is_failure(&events) { return Ok(events); }
        // Execute actions
        for action in self.actions {
            let act_events = world.execute(&action);
            let failed = Event::is_failure(&act_events);
            events.extend(act_events);
            if failed { break; }
        }

        Ok(events)
    }
}

enum Range {
    Melee,
    //Ranged { min: i32, max: i32 }
}