use std::{
    collections::HashSet,
    iter::FromIterator,
};
use hex::{self, Hex};
use log::warn;
use serde::Serialize;
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::{
    creature::{Creature, CreatureAction, Part},
    error::{Error, Result},
    event::{Action, Event, Mod, ModId, Trigger, TriggerId},
    id_map::{Id, IdMap},
    library,
    map::{Map},
    npc::{Motion},
};

#[derive(Debug, Clone)]
pub struct World {
    map: Map,
    player_id: Id<Creature>,
    creatures: IdMap<Creature>,
    mods: IdMap<Box<dyn Mod>>,
    triggers: IdMap<Box<dyn Trigger>>,
    /* TODO: persistent stat changes
    Example: effect that changes the cost of cards.
        - It can't just be a mod because that wouldn't be reflected in the UI,
          and wouldn't work with a simple action test.
        - Conclusion: stat changes (anything else?) need their own persistent category
          alongside mods and triggers.
    */
    pub tracer: Option<Box<dyn Tracer>>,
}

impl World {
    pub fn new() -> Self {
        let mut creatures = IdMap::new();
        let pc_id = creatures.add(make_player());
        let mut map = Map::new();
        map.place_at(pc_id, hex::ORIGIN).unwrap();
        let enemy_id = creatures.add(library::npc::Monopod::creature());
        map.place_at(enemy_id, Hex { x: -4, y: 1 }).unwrap();
        let enemy2_id = creatures.add(library::npc::Monopod::creature());
        map.place_at(enemy2_id, Hex { x: 4, y: -1 }).unwrap();
        let mut out = World {
            map: map,
            player_id: pc_id,
            creatures: creatures,
            mods: IdMap::new(),
            triggers: IdMap::new(),
            tracer: None,
        };
        out.update_npc_plans();
        out
    }

    // Accessors

    pub fn map(&self) -> &Map { &self.map }
    pub fn player_id(&self) -> Id<Creature> { self.player_id }
    pub fn creatures(&self) -> &IdMap<Creature> { &self.creatures }
    pub fn mods(&self) -> &IdMap<Box<dyn Mod>> { &self.mods }
    pub fn triggers(&self) -> &IdMap<Box<dyn Trigger>> { &self.triggers }

    pub fn affects_action(&self, action: &Action) -> (Vec<ModId>, Vec<TriggerId>) {
        let mods = self.mods.iter()
            .filter(|(_, m)| m.applies(action))
            .map(|(id, _)| *id)
            .collect();
        let triggers = self.triggers.iter()
            .filter(|(_, t)| t.applies(action))
            .map(|(id, _)| *id)
            .collect();
        (mods, triggers)
    }

    pub fn state(&self) -> GameState {
        let player = self.creatures.get(self.player_id).unwrap();
        if player.dead {
            return GameState::Lost;
        }
        if self.creatures.iter()
            .filter(|(&id, _)| id != self.player_id)
            .all(|(_, c)| c.dead) {
            return GameState::Won;
        }
        GameState::Play
    }

    // Mutators

    pub fn execute(&mut self, action: &Action) -> Vec<Event> {
        let mut out = vec![];
        self.execute_(action, &HashSet::new(), &mut out);
        out
    }

    pub fn move_creature(&mut self, creature_id: Id<Creature>, to: Hex) -> Vec<Event> {
        let from = match self.map.creatures().get(&creature_id) {
            Some(h) => h,
            None => return vec![Event::failed(Error::NoSuchCreature)],
        };
        let path = match self.map.path_to(*from, to) {
            Ok(p) => p,
            Err(e) => return vec![Event::failed(e)],
        };
        let mut out = vec![];
        for (from, to) in path.iter().zip(path.iter().skip(1)) {
            let actual = match self.map.creatures().get(&creature_id) {
                Some(h) => h,
                None => {
                    out.push(Event::failed(Error::NoSuchCreature));
                    return out;
                }
            };
            if actual != from && actual.distance_to(*to) > 1 {
                out.push(Event::failed(Error::Obstructed));
                return out;
            }
            let mut mp_evs = self.execute(
                &Action::ToCreature {
                    id: creature_id,
                    action: CreatureAction::SpendMP { mp: 1 },
                }
            );
            let failed = Event::is_failure(&mp_evs);
            out.append(&mut mp_evs);
            if failed { return out; }
            out.append(&mut self.execute(
                &Action::MoveCreature { id: creature_id, to: *to }
            ));
        }
        out
    }

    pub fn npc_turn(&mut self) -> Vec<Event> {
        let mut events = vec![];

        // Refill player ap/mp
        let player_id = self.player_id;
        events.extend(self.refill(player_id));
        
        // NPC turns
        let mut npc_plays = vec![];
        for (&id, creature) in &self.creatures {
            if let Some(npc) = &creature.npc {
                npc_plays.push((id, npc.next_motion.clone(), npc.next_action.clone()));
            }
        }

        for (id, motion, action) in npc_plays {
            if let Some(m) = motion {
                let r = match m {
                    Motion::ToMelee => self.move_to_melee(id),
                };
                match r {
                    Ok(es) => events.extend(es),
                    Err(e) => warn!("NPC movement failed: {}", e),
                }
            }

            if let Some(a) = action {
                match (a.run)(self, id) {
                    Ok(es) => events.extend(es),
                    Err(e) => warn!("NPC action failed: {}", e),
                }
            }
        }

        self.update_npc_plans();

        // Refill NPC ap/mp
        let refills: Vec<Id<Creature>> = self.creatures.keys().cloned()
            .filter(|&id| id != self.player_id)
            .collect();
        for id in &refills {
            events.extend(self.refill(*id));
        }

        events
    }

    // Private

    fn execute_(
        &mut self,
        action: &Action,
        skip: &HashSet<TriggerId>,
        out: &mut Vec<Event>,
    ) {
        self.tracer.as_ref().map(|t| t.start_action(action));
        let action = self.apply_mods(action);
        let events = self.resolve(&action).unwrap_or_else(|err|
            vec![Event::Failed {
                action: action.clone(),
                reason: format!("{:?}", err),
            }]
        );
        self.tracer.as_ref().map(|t| t.resolve_action(&action, &events));
        out.extend(events.clone());

        let mut trigger_ids = self.trigger_order();
        trigger_ids.reverse();
        while let Some(id) = trigger_ids.pop() {
            if skip.contains(&id) {
                continue;
            }
            let trigger = match self.triggers.get_mut(&id) {
                None => continue,
                Some(t) => t,
            };
            if !trigger.applies(&action) { continue; }
            let added: Vec<_> = events.iter().flat_map(|event| trigger.apply(&action, &event)).collect();
            let mut sub_skip = skip.clone();
            sub_skip.insert(id);
            for act in &added {
                self.execute_(act, &sub_skip, out);
            }
        }
    }

    fn trigger_order(&self) -> Vec<TriggerId> {
        // TODO: non-arbitrary order
        self.triggers.keys().cloned().collect()
    }

    fn apply_mods(&mut self, action: &Action) -> Action {
        let mut modded = action.clone();
        // TODO: non-arbitrary order
        for (_, m) in self.mods.iter_mut() {
            if !m.applies(&modded) { continue; }
            m.apply(&mut modded);
            self.tracer.as_ref().map(|t| t.mod_action(&m.name(), &modded));
        }
        modded
    }

    fn resolve(&mut self, action: &Action) -> Result<Vec<Event>> {
        use Action::*;
        match *action {
            Nothing => return Ok(vec![Event::Nothing]),
            MoveCreature { id, to } => {
                let &from = self.map.creatures().get(&id).ok_or(Error::NoSuchCreature)?;
                self.map.move_to(id, to)?;
                return Ok(vec![Event::CreatureMoved { id, from, to }]);
            }
            ToCreature { id, ref action } => {
                let creature = self.creatures.get_mut(&id).ok_or(Error::NoSuchCreature)?;
                return creature.resolve(&action).map(|cevs| {
                    cevs.into_iter().map(|cev| Event::OnCreature { id, event: cev }).collect()
                });
            },
        }
    }

    fn refill(&mut self, id: Id<Creature>) -> Vec<Event> {
        let mut events = vec![];
        let (fill_ap, fill_mp) = {
            let creature = match self.creatures.get(id) {
                Some(c) => c,
                None => return vec![],
            };
            (creature.max_ap() - creature.cur_ap, creature.max_mp() - creature.cur_mp)
        };
        if fill_ap > 0 {
            events.extend(self.execute(&Action::ToCreature {
                id, action: CreatureAction::GainAP { ap: fill_ap }
            }));
        }
        if fill_mp > 0 {
            events.extend(self.execute(&Action::ToCreature {
                id, action: CreatureAction::GainMP { mp: fill_mp }
            }));
        }
        events
    }

    fn move_to_melee(&mut self, id: Id<Creature>) -> Result<Vec<Event>> {
        let player_hex = self.map.creatures().get(&self.player_id)
            .ok_or(Error::Obstructed)?;
        let from = self.map.creatures().get(&id)
            .ok_or(Error::Obstructed)?;
        if from.distance_to(*player_hex) <= 1 { return Ok(vec![]); }
        let mut near: Vec<_> = player_hex.neighbors()
            .filter(|h| self.map.tiles().get(h).map_or(false, |t| t.is_open()))
            .collect();
        if near.is_empty() { return Err(Error::Obstructed); }
        near.sort_by(|a, b| from.distance_to(*a).cmp(&from.distance_to(*b)));
        Ok(self.move_creature(id, near[0]))
    }

    fn update_npc_plans(&mut self) {
        let ids: Vec<Id<Creature>> = self.creatures.keys().cloned().collect();
        for id in ids {
            let mut npc = {
                let creature = self.creatures.get(id).unwrap();
                match &creature.npc {
                    Some(n) => n.clone(),
                    None => continue,
                }
            };
            npc.update(self, id);
            let creature = self.creatures.get_mut(&id).unwrap();
            *creature.npc_mut().unwrap() = npc;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TsData)]
pub enum GameState {
    Play,
    Won,
    Lost,
}

pub trait Tracer: std::fmt::Debug + TracerClone {
    fn start_action(&self, action: &Action);
    fn mod_action(&self, mod_name: &str, new: &Action);
    fn resolve_action(&self, action: &Action, events: &[Event]);
}

pub trait TracerClone {
    fn clone_box(&self) -> Box<dyn Tracer>;
}

impl<T: Tracer + Clone + 'static> TracerClone for T {
    fn clone_box(&self) -> Box<dyn Tracer> { Box::new(self.clone()) }
}

impl Clone for Box<dyn Tracer> {
    fn clone(&self) -> Self { self.clone_box() }
}

fn make_player() -> Creature {
    let head = Part {
        name: "Head".into(),
        ap: 3, max_hp: 2,
        vital: true,
        ..Part::default()
    };
    let torso = Part {
        name: "Torso".into(),
        max_hp: 5,
        vital: true,
        ..Part::default()
    };
    let arm_l = Part {
        name: "Arm".into(),
        cards: IdMap::from_iter(vec![library::card::Shoot::card()]),
        max_hp: 3,
        ..Part::default()
    };
    let arm_r = arm_l.clone();
    let leg_l = Part {
        name: "Leg".into(),
        cards: IdMap::from_iter(vec![library::card::Walk::card()]),
        mp: 1, max_hp: 3,
        ..Part::default()
    };
    let leg_r = leg_l.clone();
    Creature::new(&[head, torso, arm_l, arm_r, leg_l, leg_r], None)
}