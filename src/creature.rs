#[derive(Debug, Clone)]
pub struct Creature {
    kind: Kind,
}

impl Creature {
    pub fn new(kind: Kind) -> Self {
        Creature { kind }
    }

    pub fn kind(&self) -> &Kind { &self.kind }
}

#[derive(Debug, Clone)]
pub enum Kind {
    Player {},
    NPC {},
}