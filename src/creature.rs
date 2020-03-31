use wasm_bindgen::prelude::*;

#[wasm_bindgen]
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

#[wasm_bindgen]
impl Creature {
    #[wasm_bindgen(getter)]
    pub fn player(&self) -> Option<Player> {
        match &self.kind {
            Kind::Player(p) => Some(p.clone()),
            _ => None,
        }
    }
    #[wasm_bindgen(getter)]
    pub fn npc(&self) -> Option<NPC> {
        match &self.kind {
            Kind::NPC(c) => Some(c.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Kind {
    Player(Player),
    NPC(NPC),
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Player {}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct NPC {
    #[wasm_bindgen(skip)]
    pub move_range: i32,
    #[wasm_bindgen(skip)]
    pub attack_range: i32,
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl NPC {
    #[wasm_bindgen(getter)]
    pub fn moveRange(&self) -> i32 { self.move_range }
    #[wasm_bindgen(getter)]
    pub fn attack_range(&self) -> i32 { self.attack_range }
}