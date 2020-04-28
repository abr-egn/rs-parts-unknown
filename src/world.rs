use std::{
    collections::HashSet,
};

use hex::{self, Hex};
use serde::Serialize;
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    card::{self, Target},
    creature::{Creature, CreatureAction},
    error::{Error, Result},
    event::{Action, Event},
    id_map::{Id, IdMap},
    library,
    map::{Map},
    npc::{IntentKind, Range},
    trigger::{Trigger, TriggerId},
};

#[derive(Debug, Clone)]
pub struct World {
    map: Map,
    player_id: Id<Creature>,
    creatures: IdMap<Creature>,
    triggers: IdMap<Box<dyn Trigger>>,
    pub tracer: Option<Box<dyn Tracer>>,
}

impl World {
    pub fn new() -> Self {
        let mut creatures = IdMap::new();
        let pc_id = creatures.add(library::player::player());
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
            triggers: IdMap::new(),
            tracer: None,
        };
        out.execute(
            &Action::ToCreature {
                id: pc_id,
                action: CreatureAction::NewHand,
            }
        );
        out
    }

    // Accessors

    pub fn map(&self) -> &Map { &self.map }
    pub fn player_id(&self) -> Id<Creature> { self.player_id }
    pub fn creatures(&self) -> &IdMap<Creature> { &self.creatures }
    pub fn triggers(&self) -> &IdMap<Box<dyn Trigger>> { &self.triggers }

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

    pub fn start_play(&self, creature_id: Id<Creature>, hand_ix: usize) -> Result<card::InPlay> {
        let creature = self.creatures.get(creature_id).ok_or(Error::NoSuchCreature)?;
        if hand_ix >= creature.hand.len() {
            return Err(Error::NoSuchCard);
        }
        let (part_id, card_id) = creature.hand[hand_ix];
        let part = creature.parts.get(part_id).ok_or(Error::NoSuchPart)?;
        let card = part.cards.get(card_id).ok_or(Error::NoSuchCard)?;
        let behavior = (card.start_play)(self, &creature_id, &part_id);
        Ok(card::InPlay {
            creature_id,
            part_id,
            card_id,
            behavior,
            ap_cost: card.ap_cost,
        })
    }

    // Mutators

    pub fn execute(&mut self, action: &Action) -> Vec<Event> {
        self.execute_(action, &HashSet::new())
    }

    pub fn execute_all(&mut self, actions: &[Action]) -> Vec<Event> {
        let mut out = vec![];
        for act in actions {
            let events = self.execute(&act);
            let failed = Event::is_failure(&events);
            out.extend(events);
            if failed { break; }
        }
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
                &Action::normal(
                    Action::ToCreature {
                        id: creature_id,
                        action: CreatureAction::SpendMP { mp: 1 },
                    }
                )
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

    pub fn finish_play(&mut self, in_play: card::InPlay, target: &Target) -> Vec<Event> {
        let mut events = vec![];
        events.extend(self.execute(
            &Action::ToCreature {
                id: in_play.creature_id,
                action: CreatureAction::Discard {
                    part: in_play.part_id,
                    card: in_play.card_id,
                },
            }
        ));
        events.extend(self.execute(&Action::normal(
            Action::ToCreature {
                id: in_play.creature_id,
                action: CreatureAction::SpendAP { ap: in_play.ap_cost },
            }
        )));
        if !Event::is_failure(&events) {
            events.extend(in_play.behavior.apply(self, target));
        }
        events
    }

    pub fn npc_turn(&mut self) -> Vec<Event> {
        let mut events = vec![];

        // Refill player ap/mp
        let player_id = self.player_id;
        events.extend(self.refill(player_id));

        // Refresh player hand
        events.extend(self.execute(
            &Action::ToCreature {
                id: player_id,
                action: CreatureAction::NewHand,
            }
        ));

        // Player end turn triggers
        events.extend(self.system_event(Event::PlayerTurnEnd));
        
        // NPC turns
        let mut npc_plays = vec![];
        for (&id, creature) in &self.creatures {
            if let Some(npc) = &creature.npc {
                npc_plays.push((id, npc.intent.clone()));
            }
        }

        for (id, intent) in npc_plays {
            // Motion
            // TODO: move this logic to npc.rs
            let result = match intent.kind {
                IntentKind::Attack { range: Range::Melee, .. } => self.move_to_melee(id),
                IntentKind::Stunned => Ok(vec![]),
            };
            match result {
                Ok(es) => events.extend(es),
                Err(e) => events.push(Event::FloatText {
                    on: id,
                    text: format!("{}!", e),
                }),
            }

            // Action
            match intent.check_run(self, id) {
                Ok(es) => events.extend(es),
                Err(e) => events.push(Event::FloatText {
                    on: id,
                    text: format!("{}!", e),
                }),
            }
        }

        // Refill NPC ap/mp
        let refills: Vec<Id<Creature>> = self.creatures.keys().cloned()
            .filter(|&id| id != self.player_id)
            .collect();
        for id in &refills {
            events.extend(self.refill(*id));
        }

        // NPC end turn triggers
        events.extend(self.system_event(Event::NpcTurnEnd));

        self.update_npc_plans();

        events
    }

    // Private

    fn execute_(
        &mut self,
        action: &Action,
        skip: &HashSet<TriggerId>,
    ) -> Vec<Event> {
        let mut out = vec![];
        let events = self.resolve(action).unwrap_or_else(|err|
            vec![Event::Failed {
                action: action.clone(),
                reason: format!("{:?}", err),
            }]
        );
        self.tracer.as_ref().map(|t| t.resolve_action(&action, &events));
        out.extend(events.clone());
        out.extend(self.apply_triggers(skip, &events));
        out
    }

    fn system_event(&mut self, event: Event) -> Vec<Event> {
        let mut out = vec![event.clone()];
        out.extend(self.apply_triggers(&HashSet::new(), &[event]));
        self.tracer.as_ref().map(|t| t.system_event(&out));
        out
    }

    fn apply_triggers(&mut self, skip: &HashSet<TriggerId>, events: &[Event]) -> Vec<Event> {
        let mut out = vec![];
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
            let added: Vec<_> = events.iter().flat_map(|event| trigger.apply(id, &event)).collect();
            let mut sub_skip = skip.clone();
            sub_skip.insert(id);
            for act in &added {
                out.extend(self.execute_(act, &sub_skip));
            }
        }
        out
    }

    fn trigger_order(&self) -> Vec<TriggerId> {
        let mut tmp: Vec<_> = self.triggers.iter().collect();
        tmp.sort_by(|(_, a), (_, b)| a.kind().cmp(&b.kind()));
        tmp.into_iter().map(|(id, _)| *id).collect()
    }

    fn resolve(&mut self, action: &Action) -> Result<Vec<Event>> {
        use Action::*;
        match *action {
            Nothing => return Ok(vec![Event::Nothing]),
            Normal { ref action } => {
                let out = self.resolve(&*action)?;
                let norm = out.into_iter().map(|ev| Event::Normal { event: Box::new(ev) }).collect();
                return Ok(norm)
            }
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
            }
            AddTrigger { ref trigger } => {
                let id = self.triggers.add(trigger.clone());
                return Ok(vec![Event::TriggerAdded { id }]);
            }
            RemoveTrigger { id } => {
                return Ok(if self.triggers.remove(&id).is_some() {
                    vec![Event::TriggerRemoved { id }]
                } else {
                    vec![]
                });
            }
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
            events.extend(self.execute(&Action::normal(
                Action::ToCreature {
                    id, action: CreatureAction::GainAP { ap: fill_ap }
                }
            )));
        }
        if fill_mp > 0 {
            events.extend(self.execute(&Action::normal(
                Action::ToCreature {
                    id, action: CreatureAction::GainMP { mp: fill_mp }
                }
            )));
        }
        events
    }

    fn move_to_melee(&mut self, id: Id<Creature>) -> Result<Vec<Event>> {
        let player_hex = self.map.creatures().get(&self.player_id)
            .ok_or(Error::NoSuchCreature)?;
        let from = self.map.creatures().get(&id)
            .ok_or(Error::NoSuchCreature)?;
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
    fn resolve_action(&self, action: &Action, events: &[Event]);
    fn system_event(&self, events: &[Event]);
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