use std::collections::HashMap;
use hex::Hex;
use js_sys::Array;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;
use crate::{
    card,
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
#[derive(Debug, Clone)]
pub struct World {
    wrapped: world::World,
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        World {
            wrapped: world::World::new()
        }
    }
    #[wasm_bindgen(js_name = "clone")]
    pub fn js_clone(&self) -> Self { self.clone() }

    // Accessors

    #[wasm_bindgen(getter)]
    pub fn _playerId(&self) -> JsValue {
        to_js_value(&self.wrapped.player_id())
    }

    pub fn _getTiles(&self) -> Array /* [Hex, Tile][] */ {
        self.wrapped.map().tiles().iter()
            .map(|(h, t)| {
                let tuple = Array::new();
                tuple.push(&to_js_value::<Hex>(h));
                tuple.push(&to_js_value::<Tile>(t));
                tuple
            })
            .collect()
    }

    pub fn _getTile(&self, hex: JsValue) -> JsValue /* Tile | undefined */ {
        self.wrapped.map().tiles()
            .get(&from_js_value::<Hex>(hex))
            .map_or(JsValue::undefined(), |t| to_js_value(&t))
    }

    pub fn _getCreatureMap(&self) -> Array /* [Id<Creature>, Hex][] */ {
        self.wrapped.map().creatures().iter()
            .map(|(id, hex)| {
                let tuple = Array::new();
                tuple.push(&to_js_value::<Id<creature::Creature>>(id));
                tuple.push(&to_js_value::<Hex>(hex));
                tuple
            })
            .collect()
    }

    pub fn _getCreature(&self, id: JsValue) -> JsValue {
        let id: Id<creature::Creature> = from_js_value(id);
        self.wrapped.creatures().map().get(&id)
            .map_or(JsValue::undefined(), |c| Creature::new(id, c).js())
    }

    pub fn _getCreatureHex(&self, id: JsValue) -> JsValue /* Hex | undefined */ {
        let id: Id<creature::Creature> = from_js_value(id);
        self.wrapped.map().creatures().get(&id)
            .map_or(JsValue::undefined(), to_js_value::<Hex>)
    }

    pub fn _getCreatureRange(&self, id: JsValue) -> Array /* Hex[] */ {
        let id: Id<creature::Creature> = from_js_value(id);
        let range = match self.wrapped.creatures().map().get(&id) {
            Some(c) => match c.kind() {
                creature::Kind::NPC(npc) => npc.move_range,
                _ => return Array::new(),
            },
            None => return Array::new(),
        };
        let start = match self.wrapped.map().creatures().get(&id) {
            Some(h) => h,
            None => return Array::new(),
        };
        self.wrapped.map().range_from(*start, range).into_iter()
            .map(|hex| to_js_value::<Hex>(&hex))
            .collect()
    }

    pub fn _checkSpendAP(&self, creature_id: JsValue, ap: i32) -> bool {
        let id: Id<creature::Creature> = from_js_value(creature_id);
        return self.wrapped.check_action(&Action::SpendAP { id, ap });
    }

    pub fn _startPlay(&self, card: JsValue) -> Option<Behavior> {
        let card: Card = from_js_value(card);
        let creature = self.wrapped.creatures().map().get(&card.creature_id)?;
        let part = creature.parts().map().get(&card.part_id)?;
        let real_card = part.cards.map().get(&card.id)?;
        Some(Behavior::new((real_card.start_play)(&self.wrapped, &card.creature_id)))
    }

    #[wasm_bindgen(getter)]
    pub fn logging(&self) -> bool { self.wrapped.logging }

    // Mutators

    pub fn _npcTurn(&mut self) -> Array /* Event[] */ {
        self.wrapped.npc_turn().iter()
            .map(to_js_value)
            .collect()
    }

    #[wasm_bindgen(setter)]
    pub fn set_logging(&mut self, logging: bool) {
        self.wrapped.logging = logging
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Creature {
    id: Id<creature::Creature>,
    kind: creature::Kind,
    parts: HashMap<Id<creature::Part>, Part>,
    cur_ap: i32,
}

impl Creature {
    fn new(id: Id<creature::Creature>, source: &creature::Creature) -> Creature {
        let parts = source.parts().map().iter()
            .map(|(part_id, part)| (*part_id, Part::new(*part_id, id, part)))
            .collect();
        Creature {
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
pub struct Part {
    id: Id<creature::Part>,
    creature_id: Id<creature::Creature>,
    cards: HashMap<Id<card::Card>, Card>,
    ap: i32,
}

impl Part {
    fn new(
        id: Id<creature::Part>,
        creature_id: Id<creature::Creature>,
        source: &creature::Part,
    ) -> Self {
        let cards = source.cards.map().iter()
            .map(|(card_id, card)| (*card_id, Card::new(*card_id, id, creature_id, card)))
            .collect();
        Part {
            id, creature_id, cards,
            ap: source.ap,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    id: Id<card::Card>,
    part_id: Id<creature::Part>,
    creature_id: Id<creature::Creature>,
    name: String,
    ap_cost: i32,
}

impl Card {
    fn new(
        id: Id<card::Card>,
        part_id: Id<creature::Part>,
        creature_id: Id<creature::Creature>,
        source: &card::Card,
    ) -> Self {
        Card {
            id, part_id, creature_id,
            name: source.name.clone(),
            ap_cost: source.ap_cost,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Behavior {
    wrapped: Box<dyn card::Behavior>,
}

impl Behavior {
    fn new(wrapped: Box<dyn card::Behavior>) -> Self {
        Behavior { wrapped }
    }
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl Behavior {
    pub fn _highlight(&self, world: &World, cursor: JsValue) -> Array /* Hex[] */ {
        self.wrapped.highlight(&world.wrapped, from_js_value::<Hex>(cursor)).into_iter()
            .map(|h| to_js_value::<Hex>(&h))
            .collect()
    }
    pub fn _targetValid(&self, world: &World, cursor: JsValue) -> bool {
        self.wrapped.target_valid(&world.wrapped, from_js_value::<Hex>(cursor))
    }
    pub fn _apply(&self, world: &mut World, target: JsValue) -> Array /* Event[] */ {
        let target: Hex = from_js_value(target);
        self.wrapped.apply(&mut world.wrapped, target).iter()
            .map(to_js_value)
            .collect()
    }
}