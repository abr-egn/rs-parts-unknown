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

    pub fn status_order(&self) -> Vec<StatusId> {
        let mut tmp: Vec<_> = self.status.iter().collect();
        tmp.sort_by(|(_, a), (_, b)| a.order().cmp(&b.order()));
        tmp.into_iter().map(|(id, _)| *id).collect()
    }

    pub fn apply_alters(&mut self, action: &Action) -> Action {
        let mut action = action.clone();
        let order = self.status_order();
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

    /*
    pub fn apply_triggers(&mut self, event: &Event, skip: &HashSet<StatusId>) -> Option<(StatusId, Vec<Action>)> {
        let order = self.status_order();
        for id in order {
            if skip.contains(&id) { continue; }
            let status = some_or!(self.status.get_mut(id), continue);
            let actions: Vec<_> = status.trigger(id, &event);
            if !actions.is_empty() {
                return Some((id, actions));
            }
        }
        None
    }
    */
}