use wasm_bindgen::prelude::*;

use crate::card::Card;
use crate::id_map::IdMap;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Creature {
    kind: Kind,
    parts: IdMap<Part>,
    cur_ap: i32,
}

impl Creature {
    pub fn new(kind: Kind) -> Self {
        Creature { kind, parts: IdMap::new(), cur_ap: 0 }
    }

    pub fn kind(&self) -> &Kind { &self.kind }

    pub fn cards(&self) -> impl Iterator<Item=&Card> {
        self.parts.map().values()
            .flat_map(|part| part.cards())
    }

    pub fn cur_ap(&self) -> i32 { self.cur_ap }

    pub fn max_ap(&self) -> i32 {
        self.parts.map().values().fold(0, |sum, part| sum + part.ap())
    }

    pub fn fill_ap(&mut self) {
        self.cur_ap = self.max_ap();
    }

    pub fn spend_ap(&mut self, ap: i32) -> bool {
        if ap > self.cur_ap { return false; }
        self.cur_ap -= ap;
        true
    }
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

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Part {
    cards: Vec<Card>,
    ap: i32,
    /*
    power: i32,
    max_hp: i32,
    cur_hp: i32,
    capacity: i32,
    tags: HashSet<PartTag>,
    joints: Vec<Joint>,
    */
}

impl Part {
    pub fn new(cards: Vec<Card>, ap: i32) -> Self {
        Part { cards, ap }
    }
    pub fn cards(&self) -> &[Card] { &self.cards }
    pub fn ap(&self) -> i32 { self.ap }
}

/*
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PartTag {
    Head, Torso, Limb, Arm, Leg,
    Flesh, Metal, Eldritch,
}

#[derive(Debug, Clone)]
pub struct Joint {
    required: HashSet<PartTag>,
    attached: Option<Id<Part>>,
}
*/

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
