use std::collections::HashSet;

use hex::{self, Hex};
use log::info;
use wasm_bindgen::prelude::*;

use crate::card::Walk;
use crate::creature::{self, Creature, Kind};
use crate::error::{Error, Result};
use crate::event::{Mod, Trigger, Meta, Event, Action, TriggerId};
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
        let pc_id = creatures.add(make_player());
        let mut map = Map::new();
        map.place_at(pc_id, hex::ORIGIN).unwrap();
        let npc = Kind::NPC(creature::NPC {
            move_range: 3,
            attack_range: 1,
        });
        let enemy_id = creatures.add(Creature::new(npc, &[]));
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
    let player = creature::Player::new(vec![Walk::card()]);
    let part = creature::Part {
        cards: vec![Walk::card()],
        ap: 3,
    };
    let mut pc = Creature::new(Kind::Player(player), &[part]);
    pc.fill_ap();
    pc
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

mod wasm {
    use js_sys::Array;

    use crate::wasm;

    use super::*;

    #[allow(non_snake_case)]
    #[wasm_bindgen]
    impl World {
        #[wasm_bindgen(constructor)]
        pub fn js_new() -> Self { World::new() }
    
        #[wasm_bindgen(js_name = clone)]
        pub fn js_clone(&self) -> Self { self.clone() }
    
        // Accessors
    
        #[wasm_bindgen(getter)]
        pub fn playerId(&self) -> u32 { self.player_id.value() }
    
        pub fn getTiles(&self) -> Array /* [display::Hex, Tile][] */ {
            self.map.tiles().iter()
                .map(|(h, t)| {
                    let tuple = Array::new();
                    tuple.push(&JsValue::from(wasm::Hex::new(*h)));
                    tuple.push(&JsValue::from(t.clone()));
                    tuple
                })
                .collect()
        }
    
        pub fn getTile(&self, hex: &wasm::Hex) -> Option<Tile> {
            self.map.tiles().get(&Hex { x: hex.x, y: hex.y }).cloned()
        }
    
        pub fn getCreatureMap(&self) -> Array /* [Id<Creature>, Hex][] */ {
            self.map.creatures().iter()
                .map(|(id, hex)| {
                    let tuple = Array::new();
                    tuple.push(&JsValue::from(id.value()));
                    tuple.push(&JsValue::from(wasm::Hex::new(*hex)));
                    tuple
                })
                .collect()
        }
    
        pub fn getCreature(&self, id: u32) -> Option<Creature> {
            self.creatures.map().get(&Id::synthesize(id)).cloned()
        }

        pub fn getCreatureHex(&self, id: u32) -> Option<wasm::Hex> {
            self.map.creatures().get(&Id::synthesize(id))
                .cloned()
                .map(wasm::Hex::new)
        }
    
        pub fn getCreatureRange(&self, id: u32) -> Array /* Hex[] */ {
            let id = Id::synthesize(id);
            let range = match self.creatures.map().get(&id) {
                Some(c) => match c.kind() {
                    Kind::NPC(npc) => npc.move_range,
                    _ => return Array::new(),
                },
                None => return Array::new(),
            };
            let start = match self.map.creatures().get(&id) {
                Some(h) => h,
                None => return Array::new(),
            };
            self.map.range_from(*start, range).into_iter()
                .map(wasm::Hex::new)
                .map(JsValue::from)
                .collect()
        }

        pub fn checkSpendAP(&self, creature_id: u32, ap: i32) -> bool {
            let id: Id<Creature> = Id::synthesize(creature_id);
            return self.check_action(&Action::SpendAP { id, ap });
        }
    
        // Mutators
    
        pub fn npcTurn(&mut self) -> Array /* Event[] */ {
            self.npc_turn().into_iter()
                .map(wasm::Event::new)
                .map(JsValue::from)
                .collect()
        }
    }
}