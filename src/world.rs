use std::{
    collections::HashSet,
    iter::FromIterator,
};

use enum_iterator::IntoEnumIterator;
use hex::{self, Hex};
use serde::Serialize;
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    action::{
        Action, Event, EventData, Meta, Path, Tag,
        action, event, to_creature,
    },
    creature::{Creature},
    entity::{Entity},
    error::{Error, Result},
    id_map::{Id, IdMap},
    library,
    map::{Map},
    status::{StatusDone, StatusId},
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
        let pc_id = creatures.add(library::player::player());
        map.place_at(pc_id, hex::ORIGIN).unwrap();
        let enemy_id = creatures.add(library::npc::Monopod::creature());
        map.place_at(enemy_id, Hex { x: -4, y: 1 }).unwrap();
        let enemy2_id = creatures.add(library::npc::Monopod::creature());
        map.place_at(enemy2_id, Hex { x: 4, y: -1 }).unwrap();
        let mut out = World {
            map: map,
            player_id: pc_id,
            creatures: creatures,
            entity: Entity::new(),
            tracer: None,
        };
        out.execute(&to_creature(pc_id, action::NewHand));
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

    // Mutators

    pub fn execute(&mut self, action: &Action) -> Vec<Event> {
        self.execute_(action, &HashSet::new())
    }

    pub fn npc_turn(&mut self) -> Vec<Event> {
        let mut events = vec![];

        // Refill player ap/mp
        let player_id = self.player_id;
        events.extend(self.refill(player_id));

        // Refresh player hand
        events.extend(self.execute(&to_creature(player_id, action::NewHand)));

        // Player end turn triggers
        events.extend(self.system_event(event::PlayerTurnEnd));
        
        // NPC turns
        let mut npc_plays = vec![];
        for (&id, creature) in &self.creatures {
            if creature.dead { continue; }
            if let Some(npc) = &creature.npc {
                npc_plays.push((id, npc.intent.clone()));
            }
        }

        for (id, intent) in npc_plays {
            // Motion
            match intent.move_(self, id) {
                Ok(es) => events.extend(es),
                Err(e) => events.push(to_creature(id, event::FloatText { text: format!("{}!", e) })),
            }

            // Action
            match intent.act(self, id) {
                Ok(es) => events.extend(es),
                Err(e) => events.push(to_creature(id, event::FloatText { text: format!("{}!", e) })),
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

    fn execute_(
        &mut self,
        action: &Action,
        skip: &HashSet<(Path, StatusId)>,
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

    fn entity_mut(&mut self, path: &Path) -> Result<&mut Entity> {
        match path {
            Path::World => Ok(&mut self.entity),
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
            let path = some_or!(scope.path(&action), continue);
            let entity = some_or!(self.entity_mut(&path).ok(), continue);
            action = entity.apply_alters(&path, &action);
        }

        action
    }

    fn apply_triggers(&mut self, skip: &HashSet<(Path, StatusId)>, events: &[Event]) -> Vec<Event> {
        let mut out = vec![];
        for event in events {
            let mut scoped = HashSet::new();
            for scope in Scope::into_enum_iter() {
                let path = some_or!(scope.path(event), continue);
                scoped.insert(path.clone());
                out.extend(self.apply_triggers_path(skip, event, &path));
            }
            if event.is_global() {
                for path in self.all_entity_paths() {
                    if scoped.contains(&path) { continue; }
                    out.extend(self.apply_triggers_path(skip, event, &path));
                }
            }
        }
        out
    }

    fn apply_triggers_path(&mut self, skip: &HashSet<(Path, StatusId)>, event: &Event, path: &Path) -> Vec<Event> {
        let mut out = vec![];
        let order = some_or!(self.entity_mut(&path).map(|e| e.trigger_order()).ok(), return out);
        for sid in order {
            if skip.contains(&(path.clone(), sid)) { continue; }
            let (mut actions, done) = {
                let entity = some_or!(self.entity_mut(&path).ok(), continue);
                let status = some_or!(entity.status.get_mut(sid), continue);
                status.trigger(&path, event)
            };
            if done == StatusDone::Expire {
                actions.push(Action {
                    source: Path::World,
                    target: path.clone(),
                    tags: HashSet::new(),
                    data: action::RemoveStatus { id: sid },
                });
            }
            let mut sub_skip = skip.clone();
            sub_skip.insert((path.clone(), sid));
            for action in actions {
                out.extend(self.execute_(&action, &sub_skip))
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
                let entity = self.entity_mut(&action.target)?;
                let id = entity.status.add(status.clone());
                return simple(event::StatusAdded { id });
            }
            action::RemoveStatus { id } => {
                let entity = self.entity_mut(&action.target)?;
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
        if fill_ap > 0 {
            events.extend(self.execute(&Action {
                source: Path::World,
                target: Path::Creature{ cid: id },
                tags: HashSet::from_iter(vec![Tag::NoRender]),
                data: action::GainAP { ap: fill_ap }
            }));
        }
        if fill_mp > 0 {
            events.extend(self.execute(&Action {
                source: Path::World,
                target: Path::Creature{ cid: id },
                tags: HashSet::from_iter(vec![Tag::NoRender]),
                data: action::GainMP { mp: fill_mp }
            }));
        }
        events
    }

    fn update_npc_plans(&mut self) {
        let ids: Vec<Id<Creature>> = self.creatures.keys().cloned().collect();
        for id in ids {
            let mut npc = {
                let creature = self.creatures.get(id).unwrap();
                if creature.dead { continue; }
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

    fn all_entity_paths(&self) -> Vec<Path> {
        let mut out = vec![];
        out.push(Path::World);
        for (&cid, creature) in &self.creatures {
            out.push(Path::Creature { cid });
            for &pid in creature.parts.keys() {
                out.push(Path::Part { cid, pid })
            }
        }
        out
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
pub enum Scope {
    SourcePart,
    SourceCreature,
    World,
    TargetCreature,
    TargetPart,
}

impl Scope {
    pub fn path<T>(&self, meta: &Meta<T>) -> Option<Path> {
        match self {
            Scope::SourcePart => {
                let (cid, pid) = meta.source.part()?;
                Some(Path::Part { cid, pid })
            }
            Scope::SourceCreature => {
                let cid = meta.source.creature()?;
                Some(Path::Creature { cid })
            }
            Scope::World => Some(Path::World),
            Scope::TargetCreature => {
                let cid = meta.target.creature()?;
                Some(Path::Creature { cid })
            }
            Scope::TargetPart => {
                let (cid, pid) = meta.target.part()?;
                Some(Path::Part { cid, pid })
            }
        }
    }
}