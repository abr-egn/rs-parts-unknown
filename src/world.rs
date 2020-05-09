use std::{
    collections::HashSet,
};

use enum_iterator::IntoEnumIterator;
use hex::{self, Hex};
use serde::Serialize;
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    action::{action, event, Action, Event, EventData, Meta, Path},
    card::{self, Target},
    creature::{Creature},
    entity::{Entity},
    error::{Error, Result},
    id_map::{Id, IdMap},
    //library,  TEMP
    map::{Map},
    npc::{IntentKind, Range},
    status::{Status, StatusId},
    some_or,
};

#[derive(Debug, Clone)]
pub struct World {
    map: Map,
    player_id: Id<Creature>,
    creatures: IdMap<Creature>,
    entity: Entity,
    pub tracer: Option<Box<dyn Tracer>>,
}

impl World {
    pub fn new() -> Self {
        let mut creatures = IdMap::new();
        let mut map = Map::new();
        let pc_id = Id::invalid(); // TEMP
        /* TEMP
        let pc_id = creatures.add(library::player::player());
        map.place_at(pc_id, hex::ORIGIN).unwrap();
        let enemy_id = creatures.add(library::npc::Monopod::creature());
        map.place_at(enemy_id, Hex { x: -4, y: 1 }).unwrap();
        let enemy2_id = creatures.add(library::npc::Monopod::creature());
        map.place_at(enemy2_id, Hex { x: 4, y: -1 }).unwrap();
        */
        let mut out = World {
            map: map,
            player_id: pc_id,
            creatures: creatures,
            entity: Entity::new(),
            tracer: None,
        };
        /* TEMP
        out.execute(
            &Action::ToCreature {
                id: pc_id,
                action: CreatureAction::NewHand,
            }
        );
        */
        out
    }

    // Accessors

    pub fn map(&self) -> &Map { &self.map }
    pub fn player_id(&self) -> Id<Creature> { self.player_id }
    pub fn creatures(&self) -> &IdMap<Creature> { &self.creatures }
    pub fn entity(&self) -> &Entity { &self.entity }

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

    pub fn finish_play(&mut self, in_play: card::InPlay, target: &Target) -> Vec<Event> {
        let mut events = vec![];
        /* TEMP
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
        */
        events
    }

    pub fn npc_turn(&mut self) -> Vec<Event> {
        let mut events = vec![];

        // Refill player ap/mp
        let player_id = self.player_id;
        events.extend(self.refill(player_id));

        // Refresh player hand
        /* TEMP
        events.extend(self.execute(
            &Action::ToCreature {
                id: player_id,
                action: CreatureAction::NewHand,
            }
        ));
        */

        // Player end turn triggers
        events.extend(self.system_event(event::PlayerTurnEnd));
        
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
                Err(e) => events.push(Meta {
                    source: Path::Global,
                    target: Path::Creature { cid: id },
                    tags: HashSet::new(),
                    data: event::FloatText { text: format!("{}!", e) },
                }),
            }

            // Action
            match intent.check_run(self, id) {
                Ok(es) => events.extend(es),
                Err(e) => events.push(Meta {
                    source: Path::Global,
                    target: Path::Creature { cid: id },
                    tags: HashSet::new(),
                    data: event::FloatText { text: format!("{}!", e) },
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
        events.extend(self.system_event(event::NpcTurnEnd));

        self.update_npc_plans();

        events
    }

    // Private

    fn move_creature(&mut self, creature_id: Id<Creature>, to: Hex) -> Vec<Event> {
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
            /* TEMP
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
            */
        }
        out
    }

    fn execute_(
        &mut self,
        action: &Action,
        skip: &HashSet<StatusId>,
    ) -> Vec<Event> {
        let action = self.apply_alters(action);
        let mut out = vec![];
        let events = self.resolve(&action).unwrap_or_else(|err|
            vec![Meta::new(event::Failed {
                description: format!("{:?}", err),
            })]
        );
        self.tracer.as_ref().map(|t| t.resolve_action(&action, &events));
        out.extend(events.clone());
        out.extend(self.apply_triggers(skip, &events));
        out
    }

    fn system_event(&mut self, event: EventData) -> Vec<Event> {
        let event = Meta::new(event.clone());
        let mut out = vec![event];
        out.extend(self.apply_triggers(&HashSet::new(), &out));
        self.tracer.as_ref().map(|t| t.system_event(&out));
        out
    }

    fn entity_mut<T>(&mut self, meta: &Meta<T>, scope: Scope) -> Option<&mut Entity> {
        match scope {
            Scope::SourcePart => {
                let (cid, pid) = meta.source.part()?;
                let creature = self.creatures.get_mut(cid)?;
                let part = creature.parts.get_mut(pid)?;
                Some(&mut part.entity)
            }
            Scope::SourceCreature => {
                let cid = meta.source.creature()?;
                let creature = self.creatures.get_mut(cid)?;
                Some(&mut creature.entity)
            }
            Scope::World => Some(&mut self.entity),
            Scope::TargetCreature => {
                let cid = meta.target.creature()?;
                let creature = self.creatures.get_mut(cid)?;
                Some(&mut creature.entity)
            }
            Scope::TargetPart => {
                let (cid, pid) = meta.target.part()?;
                let creature = self.creatures.get_mut(cid)?;
                let part = creature.parts.get_mut(pid)?;
                Some(&mut part.entity)
            }
        }
    }

    fn path_entity_mut(&mut self, path: &Path) -> Result<&mut Entity> {
        match path {
            Path::Global => Ok(&mut self.entity),
            Path::Creature { cid } | Path::Card { cid, .. } => {
                let creature = self.creatures.get_mut(*cid).ok_or(Error::NoSuchCreature)?;
                Ok(&mut creature.entity)
            }
            Path::Part { cid, pid } => {
                let creature = self.creatures.get_mut(*cid).ok_or(Error::NoSuchCreature)?;
                let part = creature.parts.get_mut(*pid).ok_or(Error::NoSuchPart)?;
                Ok(&mut part.entity)
            }
        }
    }

    fn apply_alters(&mut self, action: &Action) -> Action {
        let mut action = action.clone();
        for scope in Scope::into_enum_iter() {
            if let Some(entity) = self.entity_mut(&action, scope) {
                action = entity.apply_alters(&action);
            }
        }

        action
    }

    fn apply_triggers(&mut self, skip: &HashSet<StatusId>, events: &[Event]) -> Vec<Event> {
        let mut out = vec![];
        for event in events {
            for scope in Scope::into_enum_iter() {
                if let Some(order) = self.entity_mut(&event, scope).map(|e| e.status_order()) {
                    for sid in order {
                        if skip.contains(&sid) { continue; }
                        let actions = {
                            let entity = self.entity_mut(&event, scope).unwrap();
                            let status = some_or!(entity.status.get_mut(sid), continue);
                            status.trigger(sid, event)
                        };
                        let mut sub_skip = skip.clone();
                        sub_skip.insert(sid);
                        for action in actions {
                            out.extend(self.execute_(&action, &sub_skip))
                        }
                    }
                }
            }
        }
        out
    }

    fn resolve(&mut self, action: &Action) -> Result<Vec<Event>> {
        let simple = |ev| Ok(vec![action.carry(ev)]);
        match &action.data {
            // Special
            action::Nothing => return simple(event::Nothing),
            action::Fail { description } => return simple(event::Failed { description: description.clone() }),
            // Entity
            action::AddStatus { status } => {
                let entity = self.path_entity_mut(&action.target)?;
                let id = entity.status.add(status.clone());
                return simple(event::StatusAdded { id });
            }
            action::RemoveStatus { id } => {
                let entity = self.path_entity_mut(&action.target)?;
                entity.status.remove(*id).ok_or(Error::NoSuchStatus)?;
                return simple(event::StatusRemoved { id: *id });
            }
            _ => ()
        }
        match (&action.target, &action.data) {
            // Creature
            (Path::Creature { cid }, action::Move { to }) => {
                let &from = self.map.creatures().get(&cid).ok_or(Error::NoSuchCreature)?;
                self.map.move_to(*cid, *to)?;
                return simple(event::Moved { from, to: *to });
            }
            _ => ()
        }
        if let Some(cid) = action.target.creature() {
            let creature = self.creatures.get_mut(cid).ok_or(Error::NoSuchCreature)?;
            return creature.resolve(action);
        }
        Err(Error::UnhandledAction)
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
        /* TEMP
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
        */
        events
    }

    // TODO: move to npc.rs
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
            let creature = self.creatures.get_mut(id).unwrap();
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, IntoEnumIterator)]
enum Scope {
    SourcePart,
    SourceCreature,
    World,
    TargetCreature,
    TargetPart,
}