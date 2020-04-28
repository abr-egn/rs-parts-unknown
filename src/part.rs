use std::{
    collections::HashSet,
    iter::FromIterator,
};

use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    card::Card,
    creature::Creature,
    error::{Error, Result},
    event::{Action, Event},
    id_map::{Id, IdMap},
    mod_stack::{Mod, ModStack},
    world::World,
};

#[derive(Debug, Clone)]
pub struct Part {
    // Structure
    pub name: String,
    pub cards: IdMap<Card>,
    pub base_tags: HashSet<PartTag>,
    pub tag_mods: ModStack<HashSet<PartTag>>,
    // Stats
    pub max_hp: i32,
    pub cur_hp: i32,
    pub thought: i32, // action points
    pub memory: i32,  // hand size
    pub mp: i32,
    /* TODO: remaining part attributes
    power: i32,  // TODO: level?
    capacity: i32,
    joints: Vec<Joint>,
    */
}

impl Part {
    pub fn new<S: Into<String>>(name: S, tags: &[PartTag], max_hp: i32) -> Self {
        Part {
            name: name.into(),
            cards: IdMap::new(),
            base_tags: HashSet::from_iter(tags.iter().cloned()),
            tag_mods: ModStack::new(),
            thought: 0, memory: 0, mp: 0,
            max_hp, cur_hp: max_hp,
        }
    }

    pub fn tags(&self) -> HashSet<PartTag> {
        self.tag_mods.eval(self.base_tags.clone())
    }

    pub fn resolve(&mut self, action: &PartAction) -> Result<Vec<PartEvent>> {
        // TODO: allow Add/Clear TagMod
        if self.tags().contains(&PartTag::Broken) { return Err(Error::BrokenPart); }
        use PartAction::*;
        match action {
            Hit { damage } => {
                let damage = std::cmp::min(self.cur_hp, *damage);
                if damage <= 0 { return Ok(vec![]); }  // TODO: fail?
                self.cur_hp -= damage;
                let mut out = vec![PartEvent::ChangeHP { delta: -damage }];
                if self.cur_hp <= 0 {
                    if self.base_tags.remove(&PartTag::Open) {
                        out.push(PartEvent::TagsCleared { tags: vec![PartTag::Open] });
                    }
                    if self.base_tags.insert(PartTag::Broken) {
                        out.push(PartEvent::TagsSet { tags: vec![PartTag::Broken] });
                    }
                }
                return Ok(out);
            }
            SetTags { tags } => {
                let mut set = vec![];
                for tag in tags {
                    if self.base_tags.insert(*tag) {
                        set.push(*tag);
                    }
                }
                if set.is_empty() {
                    return Ok(vec![]);
                } else {
                    return Ok(vec![PartEvent::TagsSet { tags: set }]);
                }
            }
            ClearTags { tags } => {
                let mut cleared = vec![];
                for tag in tags {
                    if self.base_tags.remove(tag) {
                        cleared.push(*tag);
                    }
                }
                if cleared.is_empty() {
                    return Ok(vec![]);
                } else {
                    return Ok(vec![PartEvent::TagsCleared { tags: cleared }]);
                }
            }
            AddTagMod { m } => {
                let prev = self.tags();
                let id = self.tag_mods.add(m.clone());
                let mut out = vec![PartEvent::TagsModded { id }];
                out.extend(self.mod_delta(prev));
                Ok(out)
            }
            ClearTagMod { id } => {
                let prev = self.tags();
                self.tag_mods.remove(*id);
                let mut out = vec![PartEvent::TagsUnmodded { id: *id }];
                out.extend(self.mod_delta(prev));
                Ok(out)
            }
            Heal { hp } => {
                let hp = std::cmp::min(*hp, self.max_hp - self.cur_hp);
                if hp <= 0 { return Ok(vec![]) }  // TODO: fail?
                self.cur_hp += hp;
                Ok(vec![PartEvent::ChangeHP { delta: hp }])
            }
        }
    }

    fn mod_delta(&self, prev: HashSet<PartTag>) -> Vec<PartEvent> {
        let mut out = vec![];
        let new = self.tags();
        let added: Vec<_> = new.difference(&prev).cloned().collect();
        if !added.is_empty() {
            out.push(PartEvent::TagsSet { tags: added })
        }
        let cleared: Vec<_> = prev.difference(&new).cloned().collect();
        if !cleared.is_empty() {
            out.push(PartEvent::TagsCleared { tags: cleared })
        }
        out
    }
}

#[derive(Debug, Clone, Serialize, TsData)]
pub enum PartAction {
    Hit { damage: i32 },
    SetTags { tags: Vec<PartTag> },
    ClearTags { tags: Vec<PartTag> },
    AddTagMod {
        #[serde(skip)]
        m: TagMod
    },
    ClearTagMod { id: TagModId, },
    Heal { hp: i32 },
}

pub type TagMod = Mod<HashSet<PartTag>>;
pub type TagModId = Id<TagMod>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TsData)]
pub enum PartEvent {
    ChangeHP { delta: i32 },
    TagsSet { tags: Vec<PartTag> },
    TagsCleared { tags: Vec<PartTag> },
    TagsModded { id: TagModId },
    TagsUnmodded { id: TagModId },
}

impl PartEvent {
    pub fn tags_modded(&self) -> Option<TagModId> {
        match self {
            PartEvent::TagsModded { id } => Some(*id),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, TsData)]
pub enum PartTag {
    // State
    Vital, Broken, Open,
    // Universal: shape
    Head, Torso, Limb,
    // Universal: material
    Flesh, Machine,
    // Specialized: shape
    Arm, Leg,
}

/*
#[derive(Debug, Clone)]
pub struct Joint {
    required: HashSet<PartTag>,
    attached: Option<Id<Part>>,
}
*/

pub trait WorldExt {
    fn add_mod(&mut self, cid: Id<Creature>, pid: Id<Part>, m: TagMod) -> (TagModId, Vec<Event>);
}

impl WorldExt for World {
    fn add_mod(&mut self, cid: Id<Creature>, pid: Id<Part>, m: TagMod) -> (TagModId, Vec<Event>) {
        let events = self.execute(&Action::to_part(
            cid, pid,
            PartAction::AddTagMod { m }
        ));
        let mod_id = events[0].on_part()
            .and_then(|(_, _, event)| event.tags_modded())
            .unwrap();
        (mod_id, events)
    }
}