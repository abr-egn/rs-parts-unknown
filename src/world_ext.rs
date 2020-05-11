use std::{
    collections::HashSet,
    iter::FromIterator,
};

use crate::{
    action::{Action, Path, Tag, action},
    world::{Scope, World},
    some_or,
};

pub trait WorldExt {
    fn scale_damage<I: IntoIterator<Item=Scope>>(&self, source: &Path, target: &Path, base: i32, scopes: I) -> Option<i32>;
}

impl WorldExt for World {
    fn scale_damage<I: IntoIterator<Item=Scope>>(&self, source: &Path, target: &Path, base: i32, scopes: I) -> Option<i32> {
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
        match action.data {
            action::Hit { damage } => Some(damage),
            _ => None,
        }
    }
}