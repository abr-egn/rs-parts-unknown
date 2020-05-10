use std::collections::HashSet;

use crate::{
    action::{Action},
    id_map::IdMap,
    status::{Status, StatusId},
    some_or,
};

#[derive(Debug, Clone)]
pub struct Entity {
    pub status: IdMap<Box<dyn Status>>,
}

impl Entity {
    pub fn new() -> Self {
        Entity { status: IdMap::new() }
    }

    pub fn alter_order(&self) -> Vec<StatusId> {
        let mut tmp: Vec<_> = self.status.iter().collect();
        tmp.sort_by(|(_, a), (_, b)| a.alter_order().cmp(&b.alter_order()));
        tmp.into_iter().map(|(id, _)| *id).collect()
    }

    pub fn trigger_order(&self) -> Vec<StatusId> {
        let mut tmp: Vec<_> = self.status.iter().collect();
        tmp.sort_by(|(_, a), (_, b)| a.trigger_order().cmp(&b.trigger_order()));
        tmp.into_iter().map(|(id, _)| *id).collect()
    }

    pub fn apply_alters(&mut self, action: &Action) -> Action {
        let mut action = action.clone();
        let order = self.alter_order();
        let mut skip: HashSet<StatusId> = HashSet::new();
        'outer: loop {
            if skip.len() == order.len() { break; }
            for id in &order {
                if skip.contains(id) { continue; }
                let status = some_or!(self.status.get_mut(*id), continue);
                match status.alter(&action) {
                    Some(new) => {
                        action = new;
                        skip.insert(*id);
                        continue 'outer;
                    }
                    None => (),
                }
            }
            break;
        }

        action
    }
}