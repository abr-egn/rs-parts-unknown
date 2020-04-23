use std::{
    collections::HashSet,
    convert::TryInto,
    iter::FromIterator,
};

use rand::prelude::*;
use serde::{Deserialize, Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    card::Card,
    error::{Error, Result},
    id_map::{Id, IdMap},
    npc::{self, NPC},
    serde_empty,
    some_or,
};

pub type CardId = (Id<Part>, Id<Card>);

// TODO: stats for damage scaling
#[derive(Debug, Clone)]
pub struct Creature {
    pub name: String,
    pub parts: IdMap<Part>,
    pub cur_ap: i32,
    pub cur_mp: i32,
    pub dead: bool,
    pub npc: Option<NPC>,
    pub draw: Vec<CardId>,  // end of vec -> top of pile
    pub hand: Vec<CardId>,
    pub discard: Vec<CardId>,
}

impl Creature {
    pub fn new<S: Into<String>>(name: S, parts: &[Part], npc: Option<NPC>) -> Self {
        let mut pids = IdMap::new();
        for part in parts {
            let mut tmp = part.clone();
            tmp.cur_hp = tmp.max_hp;
            pids.add(tmp);
        }
        Creature::new_ids(name, pids, npc)
    }

    pub fn new_ids<S: Into<String>>(name: S, parts: IdMap<Part>, npc: Option<NPC>) -> Self {
        let mut out = Creature {
            name: name.into(),
            parts,
            cur_ap: 0, cur_mp: 0,
            dead: false,
            npc,
            draw: vec![], hand: vec![], discard: vec![],
        };
        out.cur_ap = out.max_ap();
        out.cur_mp = out.max_mp();
        out.reset_cards();
        out
    }

    pub fn new_npc<S: Into<String>, B: 'static + npc::Behavior>(name: S, parts: IdMap<Part>, behavior: B) -> Self {
        Creature::new_ids(name, parts, Some(NPC::new(Box::new(behavior))))
    }

    // Accessors

    pub fn max_ap(&self) -> i32 {
        self.parts.values()
            .filter(|part| !part.tags.contains(&PartTag::Broken))
            .map(|part| part.thought)
            .sum()
    }

    pub fn hand_size(&self) -> i32 {
        self.parts.values()
            .filter(|part| !part.tags.contains(&PartTag::Broken))
            .map(|part| part.memory)
            .sum()
    }

    pub fn max_mp(&self) -> i32 {
        self.parts.values()
            .filter(|part| !part.tags.contains(&PartTag::Broken))
            .map(|part| part.mp)
            .sum()
    }

    pub fn open_parts(&self) -> impl Iterator<Item=(Id<Part>, &Part)> {
        self.parts.iter()
            .map(|(id, p)| (*id, p))
            .filter(|(_, p)| p.tags.contains(&PartTag::Open))
    }

    pub fn scale_damage_from(&self, damage: i32, _part: Option<Id<Part>>) -> i32 {
        // TODO
        damage
    }

    pub fn scale_damage_to(&self, damage: i32, _part: Option<Id<Part>>) -> i32 {
        // TODO
        damage
    }

    // Mutators

    pub fn resolve(&mut self, action: &CreatureAction) -> Result<Vec<CreatureEvent>> {
        if self.dead { return Err(Error::DeadCreature); }
        use CreatureAction::*;
        use CreatureEvent::*;
        match *action {
            GainAP { ap } => {
                self.cur_ap += ap;
                return Ok(vec![ChangeAP { delta: ap }]);
            }
            SpendAP { ap } => {
                if self.cur_ap < ap { return Err(Error::NotEnough); }
                self.cur_ap -= ap;
                return Ok(vec![ChangeAP { delta: -ap }]);
            }
            GainMP { mp } => {
                self.cur_mp += mp;
                return Ok(vec![ChangeMP { delta: mp }]);
            }
            SpendMP { mp } => {
                if self.cur_mp < mp { return Err(Error::NotEnough); }
                self.cur_mp -= mp;
                return Ok(vec![ChangeMP { delta: -mp }]);
            }
            ToPart { id, ref action } => {
                let scaled_action = match action {
                    PartAction::Hit { damage } => {
                        PartAction::Hit { damage: self.scale_damage_to(*damage, Some(id)) }
                    }
                    _ => action.clone(),
                };

                let part = self.parts.get_mut(&id).ok_or(Error::NoSuchPart)?;
                let was_open = part.tags.contains(&PartTag::Open);

                let pevs = part.resolve(&scaled_action)?;

                let mut self_died = false;
                let mut out = vec![];
                // If it both became broken in this event, and as end result is broken:
                let part_broken = pevs.iter().any(|pev| match pev {
                    PartEvent::TagsSet { tags } => tags.contains(&PartTag::Broken),
                    _ => false,
                }) && part.tags.contains(&PartTag::Broken);
                out.extend(pevs.into_iter().map(|pev| CreatureEvent::OnPart { id, event: pev }));
                if part_broken {
                    if part.tags.contains(&PartTag::Vital) && !self_died {
                        self_died = true;
                        out.push(CreatureEvent::Died);
                    }
                    if self.cur_ap > self.max_ap() {
                        out.push(CreatureEvent::ChangeAP {
                            delta: self.max_ap() - self.cur_ap,
                        });
                        self.cur_ap = self.max_ap();
                    }
                    if self.cur_mp > self.max_mp() {
                        out.push(CreatureEvent::ChangeMP {
                            delta: self.max_mp() - self.cur_mp,
                        });
                        self.cur_mp = self.max_mp();
                    }
                    if was_open {
                        let ids: Vec<_> = self.parts.iter()
                            .filter_map(|(id, part)| {
                                if part.tags.contains(&PartTag::Broken) { None }
                                else { Some(*id) }
                            })
                            .collect();
                        if !ids.is_empty() {
                            let ix = thread_rng().gen_range(0, ids.len());
                            self.parts.get_mut(&ids[ix]).unwrap().tags.insert(PartTag::Open);
                        }
                    }
                }
                if self_died {
                    // TODO: event
                    self.dead = true;
                }
                Ok(out)
            }
            NewHand => {
                let mut out = vec![];
                for card in self.hand.drain(..) {
                    out.push(CreatureEvent::Discarded {
                        part: card.0, card: card.1,
                    });
                    self.discard.push(card);
                }
                let uhand: usize = self.hand_size().try_into().unwrap();
                if self.draw.len() < uhand {
                    out.push(CreatureEvent::DeckRecycled);
                    self.draw.append(&mut self.discard);
                    self.draw.shuffle(&mut rand::thread_rng());
                }
                let udraw = std::cmp::min(self.draw.len(), uhand);
                for _ in 0..udraw {
                    let card = some_or!(self.draw.pop(), break);
                    out.push(CreatureEvent::Drew { part: card.0, card: card.1 });
                    self.hand.push(card);
                }

                Ok(out)
            }
            Discard { part, card } => {
                let mut out = vec![];
                let ix = self.hand.iter().position(|&c| c == (part, card)).ok_or(Error::NoSuchCard)?;
                self.discard.push(self.hand.remove(ix));
                out.push(CreatureEvent::Discarded { part, card });
                Ok(out)
            }
        }
    }

    pub fn reset_cards(&mut self) {
        self.draw = self.parts.iter()
            .flat_map(|(&id, part)|
                part.cards.keys()
                    .map(move |&cid| (id, cid))
            ).collect();
        self.hand = vec![];
        self.discard = vec![];
    }

    // TODO: more fine-grained access
    pub fn npc_mut(&mut self) -> Option<&mut NPC> { self.npc.as_mut() }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TsData)]
pub enum CreatureAction {
    GainAP { ap: i32 },
    SpendAP { ap: i32 },
    GainMP { mp: i32 },
    SpendMP { mp: i32 },
    ToPart { id: Id<Part>, action: PartAction },
    #[serde(with = "serde_empty")]
    NewHand,
    Discard { part: Id<Part>, card: Id<Card> }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TsData)]
pub enum CreatureEvent {
    ChangeAP { delta: i32 },
    ChangeMP { delta: i32 },
    OnPart { id: Id<Part>, event: PartEvent },
    #[serde(with = "serde_empty")]
    Died,
    Discarded { part: Id<Part>, card: Id<Card> },
    Drew { part: Id<Part>, card: Id<Card> },
    #[serde(with = "serde_empty")]
    DeckRecycled,
}

#[derive(Debug, Clone)]
pub struct Part {
    // Structure
    pub name: String,
    pub cards: IdMap<Card>,
    pub tags: HashSet<PartTag>,
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
            tags: HashSet::from_iter(tags.iter().cloned()),
            thought: 0, memory: 0, mp: 0,
            max_hp, cur_hp: max_hp,
        }
    }

    pub fn resolve(&mut self, action: &PartAction) -> Result<Vec<PartEvent>> {
        if self.tags.contains(&PartTag::Broken) { return Err(Error::BrokenPart); }
        use PartAction::*;
        match action {
            Hit { damage } => {
                let damage = std::cmp::min(self.cur_hp, *damage);
                if damage <= 0 { return Ok(vec![]); }
                self.cur_hp -= damage;
                let mut out = vec![PartEvent::ChangeHP { delta: -damage }];
                if self.cur_hp <= 0 {
                    if self.tags.remove(&PartTag::Open) {
                        out.push(PartEvent::TagsCleared { tags: vec![PartTag::Open] });
                    }
                    if self.tags.insert(PartTag::Broken) {
                        out.push(PartEvent::TagsSet { tags: vec![PartTag::Broken] });
                    }
                }
                return Ok(out);
            }
            SetTags { tags } => {
                let mut set = vec![];
                for tag in tags {
                    if self.tags.insert(*tag) {
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
                    if self.tags.remove(tag) {
                        cleared.push(*tag);
                    }
                }
                if cleared.is_empty() {
                    return Ok(vec![]);
                } else {
                    return Ok(vec![PartEvent::TagsCleared { tags: cleared }]);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TsData)]
pub enum PartAction {
    Hit { damage: i32 },
    SetTags { tags: Vec<PartTag> },
    ClearTags { tags: Vec<PartTag> },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, TsData)]
pub enum PartEvent {
    ChangeHP { delta: i32 },
    TagsSet { tags: Vec<PartTag> },
    TagsCleared { tags: Vec<PartTag> },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, TsData)]
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