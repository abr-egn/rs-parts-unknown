use std::{
    collections::HashSet,
    iter::FromIterator,
};

use hex::Hex;

use crate::{
    action::{Action, Event, Path, Tag, action, to_creature},
    creature::Creature,
    id_map::Id,
    entity::Entity,
    error::{Error, Result},
    world::{Scope, World},
    some_or,
};

pub trait WorldExt {
    // Queries
    fn scale_damage<I: IntoIterator<Item=Scope>>(&self, source: &Path, target: &Path, base: i32, scopes: I) -> (Option<i32>, Action);
    fn path_entity(&self, path: &Path) -> Result<&Entity>;

    // Mutators
    fn execute_all(&mut self, actions: &[Action]) -> Vec<Event>;
    fn move_creature(&mut self, creature_id: Id<Creature>, to: Hex) -> Vec<Event>;
}

impl WorldExt for World {
    // Queries

    fn scale_damage<I: IntoIterator<Item=Scope>>(&self, source: &Path, target: &Path, base: i32, scopes: I) -> (Option<i32>, Action) {
        let mut action = Action {
            source: source.clone(),
            target: target.clone(),
            tags: HashSet::from_iter(vec![Tag::Attack]),
            data: action::Hit { damage: base },
        };
        for scope in scopes.into_iter() {
            let path = some_or!(scope.path(&action), continue);
            let entity = some_or!(self.path_entity(&path).ok(), continue);
            let mut entity = entity.clone();
            action = entity.apply_alters(&path, &action);    
        }
        let damage = match action.data {
            action::Hit { damage } => Some(damage),
            _ => None,
        };
        (damage, action)
    }

    fn path_entity(&self, path: &Path) -> Result<&Entity> {
        match path {
            Path::World => Ok(&self.entity()),
            Path::Creature { cid } | Path::Card { cid, .. } => {
                let creature = self.creatures().get(*cid).ok_or(Error::NoSuchCreature)?;
                Ok(&creature.entity)
            }
            Path::Part { cid, pid } => {
                let creature = self.creatures().get(*cid).ok_or(Error::NoSuchCreature)?;
                let part = creature.parts.get(*pid).ok_or(Error::NoSuchPart)?;
                Ok(&part.entity)
            }
        }
    }

    // Mutators

    fn execute_all(&mut self, actions: &[Action]) -> Vec<Event> {
        let mut out = vec![];
        for act in actions {
            let events = self.execute(&act);
            let failed = Event::is_failure(&events);
            out.extend(events);
            if failed { break; }
        }
        out
    }

    fn move_creature(&mut self, creature_id: Id<Creature>, to: Hex) -> Vec<Event> {
        let from = match self.map().creatures().get(&creature_id) {
            Some(h) => h,
            None => return vec![Event::failed(Error::NoSuchCreature)],
        };
        let path = match self.map().path_to(*from, to) {
            Ok(p) => p,
            Err(e) => return vec![Event::failed(e)],
        };
        let mut out = vec![];
        for (from, to) in path.iter().zip(path.iter().skip(1)) {
            let actual = match self.map().creatures().get(&creature_id) {
                Some(h) => h,
                None => {
                    out.push(Event::failed(Error::NoSuchCreature));
                    return out;
                }
            };
            if actual != from && actual.distance_to(*to) > 1 {
                out.push(Event::failed(Error::Obstructed));
                return out;
            }
            let mut mp_evs = self.execute(&Action {
                source: Path::World,
                target: Path::Creature { cid: creature_id },
                tags: HashSet::from_iter(vec![Tag::NoRender]),
                data: action::SpendMP { mp: 1 },
            });
            let failed = Event::is_failure(&mp_evs);
            out.append(&mut mp_evs);
            if failed { return out; }
            out.append(&mut self.execute(&to_creature(creature_id, action::Move { to: *to })));
        }
        out
    }
}