use std::collections::HashSet;

use hex::{self, Hex};
use serde::Serialize;

use crate::creature::Creature;
use crate::id_map::{Id, IdMap};
use crate::map::Map;
use crate::player::Player;

#[derive(Debug, Clone)]
pub struct World {
    map: Map,
    player: Player,
    creatures: IdMap<Creature>,
    mods: IdMap<Box<dyn Mod>>,
    triggers: IdMap<Box<dyn Trigger>>,
    pub logging: bool,
}

macro_rules! log {
    ($self:ident, $($args:tt)*) => {
        if $self.logging { /*godot_print!($($args)*)*/ }
    };
}

impl World {
    pub fn new() -> Self {
        let mut mods: IdMap<Box<dyn Mod>> = IdMap::new();
        mods.add(Box::new(ModDebugTag));
        let mut creatures = IdMap::new();
        let pc_id = creatures.add(Creature {});
        let mut map = Map::new();
        map.place_at(pc_id, hex::ORIGIN).unwrap();
        let enemy_id = creatures.add(Creature {});
        map.place_at(enemy_id, Hex { x: -4, y: 1 }).unwrap();
        World {
            map: map,
            player: Player::new(pc_id),
            creatures: creatures,
            mods: mods,
            triggers: IdMap::new(),
            logging: true,
        }
    }

    pub fn map(&self) -> &Map { &self.map }
    pub fn player(&self) -> &Player { &self.player }
    //pub fn creatures(&self) -> &IdMap<Creature> { &self.creatures }

    pub fn move_player(&mut self, to: Hex) -> Vec<Meta<Event>> {
        let id = self.player.creature_id();
        self.execute(&Meta::new(Action::MoveCreature { id, to }))
    }

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
        log!(self, "ACTION: {:?}", action);
        let mut modded = action.clone();
        for (_id, m) in self.mods.iter_mut() {
            let mut new = modded.clone();
            m.apply(&mut new);
            if new != modded {
                log!(self, "  [{:} ({:?})] --> {:?}", m.name(), id, new);
                modded = new;
            }
        }
        let result = Meta {
            data: self.resolve_action(&modded.data),
            tags: modded.tags.clone(),
        };
        log!(self, "  => {:?}", result);
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
