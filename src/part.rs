use std::{
    collections::{HashSet},
    iter::FromIterator,
};

use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    action::{action, event, Action, Event},
    card::Card,
    error::{Error, Result},
    entity::Entity,
    id_map::{Id, IdMap},
    mod_stack::{Mod, ModStack},
};

#[derive(Debug, Clone)]
pub struct Part {
    // Structure
    pub name: String,
    pub cards: IdMap<Card>,
    pub base_tags: HashSet<PartTag>,
    pub tag_mods: ModStack<HashSet<PartTag>>,
    pub entity: Entity,
    // Stats
    pub max_hp: i32,
    pub cur_hp: i32,
    pub thought: i32, // action points
    pub memory: i32,  // hand size
    pub mp: i32,
    /* TASK: remaining part attributes
    power: i32,  // TASK: level?
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
            entity: Entity::new(),
            thought: 0, memory: 0, mp: 0,
            max_hp, cur_hp: max_hp,
        }
    }

    pub fn tags(&self) -> HashSet<PartTag> {
        self.tag_mods.eval(self.base_tags.clone())
    }

    pub fn resolve(&mut self, action: &Action) -> Result<Vec<Event>> {
        let old_tags = self.tags();
        let mut out = self.resolve_(action)?;
        let new_tags = self.tags();
        let added: Vec<_> = new_tags.difference(&old_tags).cloned().collect();
        let removed: Vec<_> = old_tags.difference(&new_tags).cloned().collect();
        if !added.is_empty() {
            out.push(action.carry(event::TagsSet { tags: added }));
        }
        if !removed.is_empty() {
            out.push(action.carry(event::TagsCleared { tags: removed }));
        }
        Ok(out)
    }

    fn resolve_(&mut self, action: &Action) -> Result<Vec<Event>> {
        let simple = |ev| Ok(vec![action.carry(ev)]);
        match action.data {
            action::SetTags { ref tags } => {
                for tag in tags {
                    self.base_tags.insert(*tag);
                }
                return Ok(vec![]);
            }
            action::ClearTags { ref tags } => {
                for tag in tags {
                    self.base_tags.remove(tag);
                }
                return Ok(vec![]);
            }
            action::AddTagMod { ref m } => {
                let id = self.tag_mods.add(m.clone());
                return simple(event::TagsModded { id });
            }
            action::ClearTagMod { id } => {
                self.tag_mods.remove(id);
                return simple(event::TagsUnmodded { id: id });
            }
            action::Hit { damage } => {
                if self.tags().contains(&PartTag::Broken) { return Err(Error::BrokenPart); }
                let damage = std::cmp::min(self.cur_hp, damage);
                if damage <= 0 { return simple(event::ChangeHP { delta: 0 }); }
                self.cur_hp -= damage;
                if self.cur_hp <= 0 {
                    self.base_tags.remove(&PartTag::Open);
                    self.base_tags.insert(PartTag::Broken);
                }
                return simple(event::ChangeHP { delta: -damage });
            }
            action::Heal { hp } => {
                let hp = std::cmp::min(hp, self.max_hp - self.cur_hp);
                if hp <= 0 { return simple(event::ChangeHP { delta: 0 }) }
                if self.cur_hp == 0 && hp > 0 {
                    self.base_tags.remove(&PartTag::Broken);
                }
                self.cur_hp += hp;
                return simple(event::ChangeHP { delta: hp });
            }
            _ => (),
        }
        Err(Error::UnhandledAction)
    }
}

pub type TagMod = Mod<HashSet<PartTag>>;
pub type TagModId = Id<TagMod>;

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

/* TASK
#[derive(Debug, Clone)]
pub struct Joint {
    required: HashSet<PartTag>,
    attached: Option<Id<Part>>,
}
*/