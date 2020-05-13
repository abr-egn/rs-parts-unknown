use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

use hex::Hex;
use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    action::{Action, Event, Path, Tag, action},
    creature::{Creature},
    error::{Error, Result},
    id_map::Id,
    part::{Part, PartTag},
    serde_empty,
    world::World,
    some_or,
};

#[derive(Clone)]
pub struct Card {
    pub name: String,
    pub ap_cost: i32,
    // Contract: the world will not change between start_play and Behavior methods.
    pub start_play: fn(&World, &Path) -> Box<dyn Behavior>,
    pub ui: fn(&World, &Path, &Path) -> HashMap<String, String>,
}

impl Card {
    pub fn start_play(world: &World, creature_id: Id<Creature>, hand_ix: usize) -> Result<InPlay> {
        let creature = world.creatures().get(creature_id).ok_or(Error::NoSuchCreature)?;
        if hand_ix >= creature.hand.len() {
            return Err(Error::NoSuchCard);
        }
        let (part_id, card_id) = creature.hand[hand_ix];
        let part = creature.parts.get(part_id).ok_or(Error::NoSuchPart)?;
        let card = part.cards.get(card_id).ok_or(Error::NoSuchCard)?;
        let behavior = (card.start_play)(world, &Path::Part { cid: creature_id, pid: part_id });
        Ok(InPlay {
            creature_id,
            part_id,
            card_id,
            behavior,
            ap_cost: card.ap_cost,
        })
    }        
}

#[derive(Clone)]
pub struct InPlay {
    pub creature_id: Id<Creature>,
    pub part_id: Id<Part>,
    pub card_id: Id<Card>,
    pub behavior: Box<dyn Behavior>,
    pub ap_cost: i32,
}

impl InPlay {
    pub fn source(&self) -> Path {
        Path::Part { cid: self.creature_id, pid: self.part_id }
    }

    pub fn finish(self, world: &mut World, target: &Path) -> Vec<Event> {
        let mut events = world.execute(&Action {
            source: Path::World,
            target: Path::Card { cid: self.creature_id, pid: self.part_id, card: self.card_id },
            tags: HashSet::new(),
            data: action::Discard,
        });
        let ap = world.execute(&Action {
            source: Path::World,
            target: Path::Creature { cid: self.creature_id },
            tags: HashSet::from_iter(vec![Tag::NoRender]),
            data: action::SpendAP { ap: self.ap_cost },
        });
        let ap_failed = Event::is_failure(&ap);
        events.extend(ap);
        if !ap_failed {
            let source = Path::Part { cid: self.creature_id, pid: self.part_id };
            events.extend(self.behavior.apply(world, source, target.clone()));
        }
        events
    }
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Card")
            .field("name", &self.name)
            .field("ap_cost", &self.ap_cost)
            .field("start_play", &(self.start_play as usize))
            .finish()
    }
}

// TASK: power scaling
pub trait Behavior: BehaviorClone {
    fn range(&self, source: &Path, world: &World) -> Vec<Hex>;
    // TASK: allow for multiple targets
    fn target_spec(&self) -> TargetSpec;
    fn target_check(&self, world: &World, source: &Path, target: &Path) -> bool;
    fn preview(&self, world: &World, source: Path, target: Path) -> Vec<Event> {
        let mut tmp = world.clone();
        tmp.tracer = None;
        self.apply(&mut tmp, source, target)
    }
    fn apply(&self, world: &mut World, source: Path, target: Path) -> Vec<Event>;
}

impl dyn Behavior {
    pub fn target_valid(&self, world: &World, source: &Path, target: &Path) -> bool {
        if !self.target_spec().matches(world, target) { return false; }
        let range = self.range(source, world);
        if !range.is_empty() {
            let pos = some_or!(target.hex(world), return false);
            if !range.contains(&pos) { return false; }
        }
        self.target_check(world, source, target)
    }
}

// TASK: multiple targets
// May be better to just go to Parts { tags, count } rather than full
// generic Multi(Vec<TargetSpec>)
#[derive(Debug, Serialize, TsData)]
pub enum TargetSpec {
    #[serde(with = "serde_empty")]
    None,
    Part { on_player: bool, tags: Vec<Vec<PartTag>> /* Or<<X and Y>, <Q and R>> */ },
    #[serde(with = "serde_empty")]
    Creature,
}

impl TargetSpec {
    pub fn matches(&self, world: &World, target: &Path) -> bool {
        match (self, target) {
            (TargetSpec::None, Path::World) => true,
            (TargetSpec::Part { on_player, tags }, Path::Part { cid, pid }) => {
                if *on_player != (*cid == world.player_id()) { return false; }
                let creature = some_or!(world.creatures().get(*cid), return false);
                let part = some_or!(creature.parts.get(*pid), return false);
                for group in tags {
                    if group.iter().all(|tag| part.tags().contains(tag)) {
                        return true;
                    }
                }
                false
            }
            (TargetSpec::Creature, Path::Creature { cid }) => *cid != world.player_id(),
            _ => false,
        }
    }
}

pub trait BehaviorClone {
    fn clone_box(&self) -> Box<dyn Behavior>;
}

impl<T: 'static + Behavior + Clone> BehaviorClone for T {
    fn clone_box(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Behavior> {
    fn clone(&self) -> Self { self.clone_box() }
}