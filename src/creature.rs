use rand::prelude::*;

use crate::{
    action::{action, event, Action, Event, Path},
    card::Card,
    error::{Error, Result},
    entity::Entity,
    id_map::{Id, IdMap},
    npc::{NPC},
    part::{Part, PartTag},
    some_or,
};

pub type CardId = (Id<Part>, Id<Card>);

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
    pub entity: Entity,
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
            entity: Entity::new(),
        };
        out.cur_ap = out.max_ap();
        out.cur_mp = out.max_mp();
        out.reset_cards();
        out
    }

    // Accessors

    pub fn max_ap(&self) -> i32 {
        if self.dead { return 0; }
        self.parts.values()
            .filter(|part| !part.tags().contains(&PartTag::Broken))
            .map(|part| part.thought)
            .sum()
    }

    pub fn hand_size(&self) -> i32 {
        self.parts.values()
            .filter(|part| !part.tags().contains(&PartTag::Broken))
            .map(|part| part.memory)
            .sum()
    }

    pub fn max_mp(&self) -> i32 {
        if self.dead { return 0; }
        let val = self.parts.values()
            .filter(|part| !part.tags().contains(&PartTag::Broken))
            .map(|part| part.mp)
            .sum();
        std::cmp::max(val, 1)
    }

    pub fn open_parts(&self) -> impl Iterator<Item=(Id<Part>, &Part)> {
        self.parts.iter()
            .map(|(id, p)| (*id, p))
            .filter(|(_, p)| p.tags().contains(&PartTag::Open))
    }

    // Mutators

    pub fn resolve(&mut self, action: &Action) -> Result<Vec<Event>> {
        let simple = |ev| Ok(vec![action.carry(ev)]);
        if self.dead { return Err(Error::DeadCreature); }
        match action.data {
            action::GainAP { ap } => {
                self.cur_ap += ap;
                return simple(event::ChangeAP { delta: ap });
            }
            action::SpendAP { ap } => {
                if self.cur_ap < ap { return Err(Error::NotEnough("AP".into())); }
                self.cur_ap -= ap;
                return simple(event::ChangeAP { delta: -ap });
            }
            action::GainMP { mp } => {
                self.cur_mp += mp;
                return simple(event::ChangeMP { delta: mp });
            }
            action::SpendMP { mp } => {
                if self.cur_mp < mp { return Err(Error::NotEnough("MP".into())); }
                self.cur_mp -= mp;
                return simple(event::ChangeMP { delta: -mp });
            }
            action::NewHand => {
                let mut out = vec![];
                let cid = action.target.creature().unwrap();
                for card in self.hand.drain(..) {
                    let mut ev = action.carry(event::Discarded);
                    ev.target = Path::Card { cid, pid: card.0, card: card.1 };
                    out.push(ev);
                    self.discard.push(card);
                }
                for _ in 0..self.hand_size() {
                    if self.draw.is_empty() {
                        if self.discard.is_empty() {
                            return Ok(out);
                        }
                        out.push(action.carry(event::DeckRecycled));
                        self.draw.append(&mut self.discard);
                        self.draw.shuffle(&mut rand::thread_rng());
                    }
                    let card = some_or!(self.draw.pop(), break);
                    let mut ev = action.carry(event::Drew);
                    ev.target = Path::Card { cid, pid: card.0, card: card.1 };
                    out.push(ev);
                    self.hand.push(card);
                }
                return Ok(out)
            }
            _ => (),
        }
        match (&action.target, &action.data) {
            (Path::Card { pid, card, .. }, action::Discard) => {
                let ix = self.hand.iter().position(|&c| c == (*pid, *card)).ok_or(Error::NoSuchCard)?;
                self.discard.push(self.hand.remove(ix));
                return simple(event::Discarded);
            }
            _ => (),
        }
        if let Some((cid, pid)) = action.target.part() {
            let part = self.parts.get_mut(pid).ok_or(Error::NoSuchPart)?;
            let old_tags = part.tags();
            let mut out = part.resolve(action)?;
            let new_tags = part.tags();
            let mut self_died = false;
            if new_tags.difference(&old_tags).any(|t| *t == PartTag::Broken) {
                let self_ev = |ev| {
                    let mut out = action.carry(ev);
                    out.target = Path::Creature { cid };
                    out
                };
                if part.tags().contains(&PartTag::Vital) && !self_died {
                    self_died = true;
                    out.push(self_ev(event::Died));
                }
                if self.cur_ap > self.max_ap() {
                    out.push(self_ev(event::ChangeAP {
                        delta: self.max_ap() - self.cur_ap,
                    }));
                    self.cur_ap = self.max_ap();
                }
                if self.cur_mp > self.max_mp() {
                    out.push(self_ev(event::ChangeMP {
                        delta: self.max_mp() - self.cur_mp,
                    }));
                    self.cur_mp = self.max_mp();
                }
                if old_tags.contains(&PartTag::Open) {
                    let ids: Vec<_> = self.parts.iter()
                        .filter_map(|(id, part)| {
                            if part.tags().contains(&PartTag::Broken) { None }
                            else { Some(*id) }
                        })
                        .collect();
                    if !ids.is_empty() {
                        let ix = thread_rng().gen_range(0, ids.len());
                        self.parts.get_mut(ids[ix]).unwrap().base_tags.insert(PartTag::Open);
                    }
                }
            }
            if self_died {
                self.dead = true;
            }
            return Ok(out);
        }
        Err(Error::UnhandledAction)
    }

    pub fn reset_cards(&mut self) {
        self.draw = self.parts.iter()
            .flat_map(|(&id, part)|
                part.cards.keys()
                    .map(move |&cid| (id, cid))
            ).collect();
        self.draw.shuffle(&mut thread_rng());
        self.hand = vec![];
        self.discard = vec![];
    }

    // TODO: more fine-grained access
    pub fn npc_mut(&mut self) -> Option<&mut NPC> { self.npc.as_mut() }
}