use std::collections::HashMap;
use hex::Hex;
use js_sys::Array;
use serde::{Serialize, de::DeserializeOwned};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;
use crate::{
    card,
    cell::{RcCell, Ref, RefMut},
    creature,
    event::{Action},
    id_map::Id,
    map::Tile,
    world,
};

fn to_js_value<T: Serialize>(t: &T) -> JsValue { to_value(t).unwrap() }
fn from_js_value<T: DeserializeOwned>(js: JsValue) -> T { 
    from_value(js).unwrap()
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct World {
    wrapped: RcCell<world::World>,
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        World {
            wrapped: RcCell::new(world::World::new())
        }
    }
    pub fn clone(&self) -> Self {
        World {
            wrapped: RcCell::new(self.wrapped().clone())
        }
    }
    #[wasm_bindgen(getter)]
    pub fn rcCount(&self) -> usize { self.wrapped.rc_count() }
    #[wasm_bindgen(getter)]
    pub fn borrowCount(&self) -> Option<usize> { self.wrapped.borrow_count() }

    // Accessors

    #[wasm_bindgen(getter)]
    pub fn _playerId(&self) -> JsValue {
        to_js_value(&self.wrapped().player_id())
    }

    pub fn _getTiles(&self) -> Array /* [Hex, Tile][] */ {
        self.wrapped().map().tiles().iter()
            .map(|(h, t)| {
                let tuple = Array::new();
                tuple.push(&to_js_value::<Hex>(h));
                tuple.push(&to_js_value::<Tile>(t));
                tuple
            })
            .collect()
    }

    pub fn _getTile(&self, hex: JsValue) -> JsValue /* Tile | undefined */ {
        self.wrapped().map().tiles()
            .get(&from_js_value::<Hex>(hex))
            .map_or(JsValue::undefined(), |t| to_js_value(&t))
    }

    pub fn _getCreatureMap(&self) -> Array /* [Id<Creature>, Hex][] */ {
        self.wrapped().map().creatures().iter()
            .map(|(id, hex)| {
                let tuple = Array::new();
                tuple.push(&to_js_value::<Id<creature::Creature>>(id));
                tuple.push(&to_js_value::<Hex>(hex));
                tuple
            })
            .collect()
    }

    pub fn _getCreature(&self, id: JsValue) -> Option<Creature> {
        let id: Id<creature::Creature> = from_js_value(id);
        Ref::map_opt(self.wrapped(),
            |world| world.creatures().map().get(&id))
            .map(|cref| Creature { wrapped: cref })
    }

    pub fn _getSCreature(&self, id: JsValue) -> JsValue {
        let id: Id<creature::Creature> = from_js_value(id);
        self.wrapped().creatures().map().get(&id)
            .map_or(JsValue::undefined(), |c| SCreature::new(id, c).js())
    }

    pub fn _getCreatureHex(&self, id: JsValue) -> JsValue /* Hex | undefined */ {
        let id: Id<creature::Creature> = from_js_value(id);
        self.wrapped().map().creatures().get(&id)
            .map_or(JsValue::undefined(), to_js_value::<Hex>)
    }

    pub fn _getCreatureRange(&self, id: JsValue) -> Array /* Hex[] */ {
        let id: Id<creature::Creature> = from_js_value(id);
        let wrapped = self.wrapped();
        let range = match wrapped.creatures().map().get(&id) {
            Some(c) => match c.kind() {
                creature::Kind::NPC(npc) => npc.move_range,
                _ => return Array::new(),
            },
            None => return Array::new(),
        };
        let start = match wrapped.map().creatures().get(&id) {
            Some(h) => h,
            None => return Array::new(),
        };
        wrapped.map().range_from(*start, range).into_iter()
            .map(|hex| to_js_value::<Hex>(&hex))
            .collect()
    }

    pub fn _checkSpendAP(&self, creature_id: JsValue, ap: i32) -> bool {
        let id: Id<creature::Creature> = from_js_value(creature_id);
        return self.wrapped().check_action(&Action::SpendAP { id, ap });
    }

    pub fn _startPlay(&self, creature_id: JsValue, part_id: JsValue, card_id: JsValue) -> Behavior {
        let creature_id: Id<creature::Creature> = from_js_value(creature_id);
        let part_id: Id<creature::Part> = from_js_value(part_id);
        let card_id: Id<card::Card> = from_js_value(card_id);
        let world = self.wrapped();
        let creature = world.creatures().map().get(&creature_id).unwrap();
        let part = creature.parts().map().get(&part_id).unwrap();
        let card = part.cards.map().get(&card_id).unwrap();
        Behavior::new((card.start_play)(&world, &creature_id))
    }

    #[wasm_bindgen(getter)]
    pub fn logging(&self) -> bool { self.wrapped().logging }

    // Mutators

    pub fn _npcTurn(&mut self) -> Array /* Event[] */ {
        self.wrapped_mut().npc_turn().iter()
            .map(to_js_value)
            .collect()
    }

    #[wasm_bindgen(setter)]
    pub fn set_logging(&mut self, logging: bool) {
        self.wrapped_mut().logging = logging
    }
}

impl World {
    pub fn wrapped(&self) -> Ref<world::World> { self.wrapped.borrow() }
    pub fn wrapped_mut(&mut self) -> RefMut<world::World> { self.wrapped.borrow_mut() }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SCreature {
    id: Id<creature::Creature>,
    kind: creature::Kind,
    parts: HashMap<Id<creature::Part>, SPart>,
    cur_ap: i32,
}

impl SCreature {
    fn new(id: Id<creature::Creature>, source: &creature::Creature) -> SCreature {
        let parts = source.parts().map().iter()
            .map(|(part_id, part)| (*part_id, SPart::new(*part_id, id, part)))
            .collect();
        SCreature {
            id,
            kind: source.kind.clone(),
            parts,
            cur_ap: source.cur_ap,
        }
    }
    fn js(&self) -> JsValue { to_js_value(&self) }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SPart {
    id: Id<creature::Part>,
    creature_id: Id<creature::Creature>,
    cards: HashMap<Id<card::Card>, SCard>,
    ap: i32,
}

impl SPart {
    fn new(
        id: Id<creature::Part>,
        creature_id: Id<creature::Creature>,
        source: &creature::Part,
    ) -> Self {
        let cards = source.cards.map().iter()
            .map(|(card_id, card)| (*card_id, SCard::new(*card_id, id, creature_id, card)))
            .collect();
        SPart {
            id, creature_id, cards,
            ap: source.ap,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SCard {
    id: Id<card::Card>,
    part_id: Id<creature::Part>,
    creature_id: Id<creature::Creature>,
    name: String,
    ap_cost: i32,
}

impl SCard {
    fn new(
        id: Id<card::Card>,
        part_id: Id<creature::Part>,
        creature_id: Id<creature::Creature>,
        source: &card::Card,
    ) -> Self {
        SCard {
            id, part_id, creature_id,
            name: source.name.clone(),
            ap_cost: source.ap_cost,
        }
    }
}

#[wasm_bindgen]
pub struct Creature {
    wrapped: Ref<world::World, creature::Creature>,
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl Creature {
    pub fn _player(&self) -> Option<Player> {
        let tmp = Ref::clone(&self.wrapped);
        Ref::map_opt(tmp, |creature| {
            match creature.kind {
                creature::Kind::Player(ref p) => Some(p),
                _ => None,
            }
        }).map(|xp| Player { _wrapped: xp })
    }
    pub fn _npc(&self) -> JsValue /* NPC | undefined */ {
        match self.wrapped.kind() {
            creature::Kind::NPC(c) => to_js_value::<creature::NPC>(c),
            _ => JsValue::undefined(),
        }
    }
    pub fn curAP(&self) -> i32 { self.wrapped.cur_ap() }
    pub fn maxAP(&self) -> i32 { self.wrapped.max_ap() }
    pub fn _cards(&self) -> Array /* Card[] */ {
        self.wrapped.cards()
            .map(|(pid, cid, _)| {
                let tmp = Ref::clone(&self.wrapped);
                Card {
                    wrapped: Ref::map(tmp, |cref| {
                        cref.parts().map().get(&pid).unwrap().cards.map().get(&cid).unwrap()
                    }),
                }
            })
            .map(JsValue::from)
            .collect()
    }
}

#[wasm_bindgen]
pub struct Player {
    _wrapped: Ref<world::World, crate::creature::Player>,
}

#[wasm_bindgen]
pub struct Card {
    wrapped: Ref<world::World, card::Card>,
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl Card {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String { self.wrapped.name.clone() }
    #[wasm_bindgen(getter)]
    pub fn apCost(&self) -> i32 { self.wrapped.ap_cost }
    pub fn _startPlay(&self, world: &World, source: JsValue) -> Behavior {
        let source: Id<creature::Creature> = from_js_value(source);
        Behavior::new((self.wrapped.start_play)(&world.wrapped(), &source))
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Behavior {
    wrapped: Box<dyn card::Behavior>,
}

impl Behavior {
    pub fn new(wrapped: Box<dyn card::Behavior>) -> Self {
        Behavior { wrapped }
    }
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl Behavior {
    pub fn highlight(&self, world: &World, cursor: JsValue) -> Array /* Hex[] */ {
        self.wrapped.highlight(&world.wrapped.borrow(), from_js_value::<Hex>(cursor)).into_iter()
            .map(|h| to_js_value::<Hex>(&h))
            .collect()
    }
    pub fn targetValid(&self, world: &World, cursor: JsValue) -> bool {
        self.wrapped.target_valid(&world.wrapped.borrow(), from_js_value::<Hex>(cursor))
    }
    pub fn apply(&self, world: &mut World, target: JsValue) -> Array /* Event[] */ {
        let mut world = world.wrapped.borrow_mut();
        let target: Hex = from_js_value(target);
        self.wrapped.apply(&mut world, target).iter()
            .map(to_js_value)
            .collect()
    }
}