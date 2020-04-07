use std::collections::HashSet;

use hex::{self, Hex};
use log::info;

use crate::card::Walk;
use crate::creature::{self, Creature};
use crate::error::{Error, Result};
use crate::event::{Mod, Trigger, Meta, Event, Action, TriggerId};
use crate::id_map::{Id, IdMap};
use crate::map::{Tile, Map, Space};

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
        let pc_id = creatures.add(make_player());
        let mut map = Map::new();
        map.place_at(pc_id, hex::ORIGIN).unwrap();
        let enemy_id = creatures.add(make_npc());
        map.place_at(enemy_id, Hex { x: -4, y: 1 }).unwrap();
        let enemy2_id = creatures.add(make_npc());
        map.place_at(enemy2_id, Hex { x: 4, y: -1 }).unwrap();
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
    pub fn creatures(&self) -> &IdMap<Creature> { &self.creatures }

    pub fn check_action(&self, action: &Action) -> bool {
        let mut check = self.clone();
        check.logging = false;
        let result = check.execute(&Meta::new(action.clone()));
        // First event is always the originating action
        match result[0].data {
            Event::Failed { .. } => return false,
            _ => return true,
        }
    }

    // Mutators

    pub fn execute(&mut self, action: &Meta<Action>) -> Vec<Meta<Event>> {
        let mut out = vec![];
        self.execute_(action, &HashSet::new(), &mut out);
        out
    }

    pub fn npc_turn(&mut self) -> Vec<Meta<Event>> {
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

    fn execute_(
        &mut self,
        action: &Meta<Action>,
        skip: &HashSet<TriggerId>,
        out: &mut Vec<Meta<Event>>,
    ) {
        let event = self.resolve_with_mods(action);
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

    fn resolve_with_mods(&mut self, action: &Meta<Action>) -> Meta<Event> {
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
            data: self.resolve_action(&modded.data).unwrap_or_else(|err|
                Event::Failed {
                    action: modded.data.clone(),
                    reason: format!("{:?}", err),
                }
            ),
            tags: modded.tags.clone(),
        };
        clog!(self, "  => {:?}", result);
        result
    }

    fn resolve_action(&mut self, action: &Action) -> Result<Event> {
        use Action::*;
        match *action {
            Nothing => return Ok(Event::Nothing),
            MoveCreature { id, to } => {
                let &from = self.map.creatures().get(&id).ok_or(Error::NoSuchCreature)?;
                self.map.move_to(id, to)?;
                return Ok(Event::CreatureMoved { id, from, to });
            }
            SpendAP { id, ap } => {
                let creature = self.creatures.get_mut(&id).ok_or(Error::NoSuchCreature)?;
                if creature.spend_ap(ap) {
                    return Ok(Event::SpentAP { id, ap })
                } else {
                    return Err(Error::NotEnoughAP)
                }
            }
        }
    }
}

fn make_player() -> Creature {
    let mut cards = IdMap::new();
    cards.add(Walk::card());
    let part = creature::Part { cards, ap: 3 };
    let mut pc = Creature::new(&[part]);
    pc.cur_ap = pc.max_ap();
    pc.cur_mp = 1;
    pc
}

fn make_npc() -> Creature {
    let mut npc = Creature::new(&[]);
    npc.cur_mp = 3;
    npc
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