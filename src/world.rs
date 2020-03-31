use std::collections::HashSet;

use hex::{self, Hex};
use js_sys::Array;
use log::info;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::creature::{Creature, Kind};
use crate::display;
use crate::id_map::{Id, IdMap};
use crate::map::{Tile, Map, Space};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct World {
    map: Map,
    player_id: Id<Creature>,
    creatures: IdMap<Creature>,
    mods: IdMap<Box<dyn Mod>>,
    triggers: IdMap<Box<dyn Trigger>>,
    pub logging: bool,
}

macro_rules! clog {
    ($self:ident, $($args:tt)*) => {
        if $self.logging { info!($($args)*) }
    };
}

impl World {
    pub fn new() -> Self {
        let mut mods: IdMap<Box<dyn Mod>> = IdMap::new();
        mods.add(Box::new(ModDebugTag));
        let mut creatures = IdMap::new();
        let pc_id = creatures.add(Creature::new(Kind::Player {}));
        let mut map = Map::new();
        map.place_at(pc_id, hex::ORIGIN).unwrap();
        let enemy_id = creatures.add(Creature::new(Kind::NPC {}));
        map.place_at(enemy_id, Hex { x: -4, y: 1 }).unwrap();
        World {
            map: map,
            player_id: pc_id,
            creatures: creatures,
            mods: mods,
            triggers: IdMap::new(),
            logging: true,
        }
    }

    // Accessors

    pub fn map(&self) -> &Map { &self.map }
    pub fn player_id(&self) -> Id<Creature> { self.player_id }
    //pub fn creatures(&self) -> &IdMap<Creature> { &self.creatures }

    // Mutators

    pub fn move_player(&mut self, to: Hex) -> Vec<Meta<Event>> {
        self.execute(&Meta::new(Action::MoveCreature { id: self.player_id, to }))
    }

    pub fn end_turn(&mut self) -> Vec<Meta<Event>> {
        let player_hex = self.map.creatures().get(&self.player_id).unwrap();

        let mut moves = vec![];
        for &id in self.creatures.map().keys() {
            if id == self.player_id { continue }
            let hex = match self.map.creatures().get(&id) {
                Some(v) => v,
                None => continue,
            };
            let mut neighbors: Vec<_> = hex.neighbors()
                .filter(|n| match self.map.tiles().get(n) {
                    Some(Tile { space: Space::Empty, creature: None }) => true,
                    _ => false,
                })
                .collect();
            neighbors.sort_by(|a, b|
                player_hex.distance_to(*a).cmp(&player_hex.distance_to(*b))
            );
            if let Some(&to) = neighbors.get(0) {
                moves.push((id, to));
            }
        }

        let mut events = vec![];
        for (id, to) in moves {
            events.extend(self.execute(&Meta::new(
                Action::MoveCreature { id, to }
            )));
        }
        events
    }

    // Private

    fn execute(&mut self, action: &Meta<Action>) -> Vec<Meta<Event>> {
        let mut out = vec![];
        self.execute_(action, &HashSet::new(), &mut out);
        out
    }

    fn execute_(
        &mut self,
        action: &Meta<Action>,
        skip: &HashSet<TriggerId>,
        out: &mut Vec<Meta<Event>>,
    ) {
        let event = self.resolve_mods(action);
        out.push(event.clone());
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
            let added = trigger.apply(&event);
            let mut sub_skip = skip.clone();
            sub_skip.insert(id);
            for act in &added {
                self.execute_(act, &sub_skip, out);
            }
        }
    }

    fn trigger_order(&self) -> Vec<TriggerId> {
        // TODO: non-arbitrary
        self.triggers.map().keys().cloned().collect()
    }

    fn resolve_mods(&mut self, action: &Meta<Action>) -> Meta<Event> {
        clog!(self, "ACTION: {:?}", action);
        let mut modded = action.clone();
        for (id, m) in self.mods.iter_mut() {
            let mut new = modded.clone();
            m.apply(&mut new);
            if new != modded {
                clog!(self, "  [{:} ({:?})] --> {:?}", m.name(), id, new);
                modded = new;
            }
        }
        let result = Meta {
            data: self.resolve_action(&modded.data),
            tags: modded.tags.clone(),
        };
        clog!(self, "  => {:?}", result);
        result
    }

    fn resolve_action(&mut self, action: &Action) -> Event {
        use Action::*;
        match *action {
            MoveCreature { id, to } => match self.map.move_to(id, to) {
                Ok(path) => Event::CreatureMoved { id, path },
                Err(_) => Event::Failed { action: action.clone(), reason: String::from("??") },
            }
        }
    }
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl World {
    #[wasm_bindgen(getter)]
    pub fn playerId(&self) -> u32 { self.player_id.value() }

    pub fn getTiles(&self) -> Array /* [display::Hex, Tile][] */ {
        self.map.tiles().iter()
            .map(|(h, t)| {
                let tuple = Array::new();
                tuple.push(&JsValue::from(display::Hex::new(h)));
                tuple.push(&JsValue::from(t.clone()));
                tuple
            })
            .collect()
    }

    pub fn getTile(&self, hex: display::Hex) -> Option<Tile> {
        self.map.tiles().get(&Hex { x: hex.x, y: hex.y }).cloned()
    }

    pub fn getCreatures(&self) -> Array /* [display::Hex, display::Creature][] */ {
        let out = Array::new();

        for (id, hex) in self.map.creatures() {
            let tuple = Array::new();
            tuple.push(&JsValue::from(id.value()));
            tuple.push(&JsValue::from(self.new_creature(id, hex)));
            out.push(&JsValue::from(tuple));
        }

        out
    }

    pub fn getCreature(&self, id: u32) -> Option<display::Creature> {
        let id: Id<Creature> = Id::synthesize(id);
        self.map.creatures().get(&id).map(|hex| self.new_creature(&id, hex))
    }

    fn new_creature(&self, id: &Id<Creature>, hex: &Hex) -> display::Creature {
        let label = String::from(if *id == self.player_id { "P" } else { "X" });
        display::Creature {
            hex: display::Hex::new(hex),
            label,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Meta<T> {
    data: T,
    tags: HashSet<String>,
}

impl<T> Meta<T> {
    pub fn new(data: T) -> Self {
        Meta {
            data,
            tags: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Action {
    MoveCreature { id: Id<Creature>, to: Hex },
}

#[derive(Debug, Clone, Serialize)]
pub enum Event {
    CreatureMoved { id: Id<Creature>, path: Vec<Hex>, },
    Failed { action: Action, reason: String },
}

pub trait Mod: ModClone + std::fmt::Debug + Send {
    fn name(&self) -> &'static str;
    fn apply(&mut self, action: &mut Meta<Action>);
}

pub trait ModClone {
    fn clone_box(&self) -> Box<dyn Mod>;
}

impl<T> ModClone for T
where
    T: 'static + Mod + Clone,
{
    fn clone_box(&self) -> Box<dyn Mod> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Mod> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone, Debug)]
struct ModDebugTag;

impl Mod for ModDebugTag {
    fn name(&self) -> &'static str {
        "debug tag"
    }
    fn apply(&mut self, action: &mut Meta<Action>) {
        action.tags.insert("debug".into());
    }
}

pub trait Trigger: TriggerClone + std::fmt::Debug + Send {
    fn name(&self) -> &'static str;
    fn apply(&mut self, event: &Meta<Event>) -> Vec<Meta<Action>>;
}

type TriggerId = Id<Box<dyn Trigger>>;

pub trait TriggerClone {
    fn clone_box(&self) -> Box<dyn Trigger>;
}

impl<T> TriggerClone for T
where
    T: 'static + Trigger + Clone,
{
    fn clone_box(&self) -> Box<dyn Trigger> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Trigger> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
