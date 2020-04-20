use crate::{
    creature::{Creature, CreatureAction, Part, PartTag},
    error::{Error, Result},
    event::{Action, Event},
    id_map::{Id},
    npc::{self, NPC},
    world::World,
};

#[derive(Debug, Clone)]
pub struct Monopod {
    kick_time: bool,
}

impl npc::Behavior for Monopod {
    fn next(&mut self, _world: &World, _id: Id<Creature>) -> (Option<npc::Motion>, Option<npc::Action>) {
        let action = if self.kick_time {
            Some(npc::Action {
                kind: npc::ActionKind::Attack,
                run: Monopod::kick,
            })
        } else { None };
        self.kick_time = !self.kick_time;
        (Some(npc::Motion::ToMelee), action)
    }
}

impl Monopod {
    pub fn creature() -> Creature {
        let head = Part {
            thought: 1,
            ..Part::new("Hed", &[PartTag::Vital], 2)
        };
        let foot = Part {
            mp: 3,
            ..Part::new("Fut", &[], 2)
        };
        Creature::new(&[head, foot], Some(Monopod::npc()))
    }
    fn npc() -> NPC {
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
            actions: vec![Action::ToCreature {
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
    actions: Vec<Action>,
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
        if !creature.parts.values().any(|p| p.name == part && !p.tags.contains(&PartTag::Broken)) {
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
        let mut events = world.execute(&Action::ToCreature {
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