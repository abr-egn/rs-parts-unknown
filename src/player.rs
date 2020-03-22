use crate::creature::Creature;
use crate::id_map::Id;

#[derive(Debug, Clone)]
pub struct Player {
    creature_id: Id<Creature>,
}

impl Player {
    pub fn new(creature_id: Id<Creature>) -> Self {
        Player { creature_id }
    }
    pub fn creature_id(&self) -> Id<Creature> {
        self.creature_id
    }
}
