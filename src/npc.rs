use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    creature::{Creature},
    error::{Error, Result},
    action::{Action, Event},
    id_map::Id,
    part::{Part, PartTag},
    serde_empty,
    world::World,
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
    pub fn check_run(&self, world: &mut World, source: Id<Creature>) -> Result<Vec<Event>> {
        self.check(world, source)?;
        self.kind.check(world, source)?;
        let mut events = vec![];

        // Execute cost
        /* TEMP
        let mut events = world.execute(&Action::normal(
            Action::ToCreature {
                id: source,
                action: CreatureAction::SpendAP { ap: 1 },
            }
        ));
        if Event::is_failure(&events) { return Ok(events); }
        */

        // Execute action
        events.extend(self.kind.run(world, source, self.from));

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

    fn run(&self, world: &mut World, source: Id<Creature>, _part: Option<Id<Part>>) -> Vec<Event> {
        /* TEMP
        match self {
            IntentKind::Attack { damage, .. } => {
                let player_id = world.player_id();
                let player = world.creatures().get(player_id).unwrap();
                let mut open: Vec<_> = player.open_parts().collect();
                if open.is_empty() { return vec![]; }
                open.sort_by(|(_, a), (_, b)| a.cur_hp.cmp(&b.cur_hp));
                let (pid, _) = open.first().unwrap();
                let hit = Action::to_part(player_id, *pid, PartAction::Hit { damage: *damage });
                world.execute(&hit)
            }
            IntentKind::Stunned => vec![Event::FloatText { on: source, text: "Stunned!".into() }]
        }
        */
        unimplemented!()
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