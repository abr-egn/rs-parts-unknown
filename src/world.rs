use std::collections::HashSet;
use hex::{self, Hex};
use crate::{
    card::Walk,
    creature::{self, Creature},
    error::{Error, Result},
    event::{Action, Event, Mod, ModId, Trigger, TriggerId},
    id_map::{Id, IdMap},
    map::{Tile, Map, Space},
};

#[derive(Debug, Clone)]
pub struct World {
    map: Map,
    player_id: Id<Creature>,
    creatures: IdMap<Creature>,
    mods: IdMap<Box<dyn Mod>>,
    triggers: IdMap<Box<dyn Trigger>>,
    /* TODO
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
        let enemy_id = creatures.add(make_npc());
        map.place_at(enemy_id, Hex { x: -4, y: 1 }).unwrap();
        let enemy2_id = creatures.add(make_npc());
        map.place_at(enemy2_id, Hex { x: 4, y: -1 }).unwrap();
        World {
            map: map,
            player_id: pc_id,
            creatures: creatures,
            mods: IdMap::new(),
            triggers: IdMap::new(),
            tracer: None,
        }
    }

    // Accessors

    pub fn map(&self) -> &Map { &self.map }
    pub fn player_id(&self) -> Id<Creature> { self.player_id }
    pub fn creatures(&self) -> &IdMap<Creature> { &self.creatures }

    // TODO(random)
    pub fn check_action(&self, action: &Action) -> bool {
        let mut check = self.clone();
        check.tracer = None;
        !Event::is_failure(&check.execute(action))
    }

    pub fn affects_action(&self, action: &Action) -> (Vec<ModId>, Vec<TriggerId>) {
        let mods = self.mods.map().iter()
            .filter(|(_, m)| m.applies(action))
            .map(|(id, _)| *id)
            .collect();
        let triggers = self.triggers.map().iter()
            .filter(|(_, t)| t.applies(action))
            .map(|(id, _)| *id)
            .collect();
        (mods, triggers)
    }

    // Mutators

    pub fn execute(&mut self, action: &Action) -> Vec<Event> {
        let mut out = vec![];
        self.execute_(action, &HashSet::new(), &mut out);
        out
    }

    pub fn npc_turn(&mut self) -> Vec<Event> {
        let mut events = vec![];

        // Refill player ap/mp
        let player_id = self.player_id;
        events.extend(self.refill(&player_id));
        
        // Move NPCs
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
        for (id, to) in moves {
            events.extend(self.execute(&Action::MoveCreature { id, to }));
        }

        // Refill NPC ap/mp
        let refills: Vec<Id<Creature>> = self.creatures.map().keys().cloned()
            .filter(|&id| id != self.player_id)
            .collect();
        for id in &refills {
            events.extend(self.refill(id));
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
        let action = self.resolve_mods(action);
        let event = self.resolve_action(&action).unwrap_or_else(|err|
            Event::Failed {
                action: action.clone(),
                reason: format!("{:?}", err),
            }
        );
        self.tracer.as_ref().map(|t| t.resolve_action(&action, &event));
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
            if !trigger.applies(&action) { continue; }
            let added = trigger.apply(&action, &event);
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

    fn resolve_mods(&mut self, action: &Action) -> Action {
        let mut modded = action.clone();
        for (_, m) in self.mods.iter_mut() {
            if !m.applies(&modded) { continue; }
            m.apply(&mut modded);
            self.tracer.as_ref().map(|t| t.mod_action(&m.name(), &modded));
        }
        modded
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
            GainAP { id, ap } => {
                let creature = self.creatures.get_mut(&id).ok_or(Error::NoSuchCreature)?;
                creature.cur_ap += ap;
                return Ok(Event::ChangeAP { id, ap })
            }
            SpendAP { id, ap } => {
                let creature = self.creatures.get_mut(&id).ok_or(Error::NoSuchCreature)?;
                if creature.spend_ap(ap) {
                    return Ok(Event::ChangeAP { id, ap: -ap })
                } else {
                    return Err(Error::NotEnough)
                }
            }
            GainMP { id, mp } => {
                let creature = self.creatures.get_mut(&id).ok_or(Error::NoSuchCreature)?;
                creature.cur_mp += mp;
                return Ok(Event::ChangeMP { id, mp })
            }
            SpendMP { id, mp } => {
                let creature = self.creatures.get_mut(&id).ok_or(Error::NoSuchCreature)?;
                if creature.spend_mp(mp) {
                    return Ok(Event::ChangeMP { id, mp: -mp })
                } else {
                    return Err(Error::NotEnough)
                }
            }
        }
    }

    fn refill(&mut self, id: &Id<Creature>) -> Vec<Event> {
        let mut events = vec![];
        let (fill_ap, fill_mp) = {
            let creature = match self.creatures.map().get(id) {
                Some(c) => c,
                None => return vec![],
            };
            (creature.max_ap() - creature.cur_ap, creature.max_mp() - creature.cur_mp)
        };
        if fill_ap > 0 {
            events.extend(self.execute(&Action::GainAP { id: *id, ap: fill_ap }));
        }
        if fill_mp > 0 {
            events.extend(self.execute(&Action::GainMP { id: *id, mp: fill_mp }));
        }
        events
    }
}

pub trait Tracer: std::fmt::Debug + TracerClone {
    fn start_action(&self, action: &Action);
    fn mod_action(&self, mod_name: &str, new: &Action);
    fn resolve_action(&self, action: &Action, event: &Event);
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
    let mut cards = IdMap::new();
    cards.add(Walk::card());
    let part = creature::Part { cards, ap: 3, mp: 2 };
    let mut pc = Creature::new(&[part]);
    pc.cur_ap = pc.max_ap();
    pc.cur_mp = 2;
    pc
}

fn make_npc() -> Creature {
    let mut npc = Creature::new(&[]);
    npc.cur_mp = 3;
    npc
}