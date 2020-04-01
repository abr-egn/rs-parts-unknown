use wasm_bindgen::prelude::*;

use crate::card::Card;

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


#[derive(Debug, Clone)]
pub enum Kind {
    Player(Player),
    NPC(NPC),
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Player {
    cards: Vec<Card>,
}

impl Player {
    pub fn new(cards: Vec<Card>) -> Self {
        Player { cards }
    }

    pub fn cards(&self) -> &[Card] { &self.cards }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct NPC {
    #[wasm_bindgen(skip)]
    pub move_range: i32,
    #[wasm_bindgen(skip)]
    pub attack_range: i32,
}

mod wasm {
    use js_sys::Array;
    use wasm_bindgen::prelude::*;

    use super::*;

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

    #[allow(non_snake_case)]
    #[wasm_bindgen]
    impl NPC {
        #[wasm_bindgen(getter = moveRange)]
        pub fn move_range(&self) -> i32 { self.move_range }
        #[wasm_bindgen(getter = attackRange)]
        pub fn attack_range(&self) -> i32 { self.attack_range }
    }

    #[wasm_bindgen]
    impl Player {
        #[wasm_bindgen(getter = cards)]
        pub fn js_cards(&self) -> Array /* Card[] */ {
            self.cards().iter().cloned()
                .map(JsValue::from)
                .collect()
        }
    }
}
