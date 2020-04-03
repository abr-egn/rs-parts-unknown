use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use hex::Hex;
use js_sys::Array;
use serde::{Serialize, de::DeserializeOwned};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

use crate::card;
use crate::creature::{Creature, Kind};
use crate::event::{self, Action};
use crate::id_map::Id;
use crate::map::Tile;
use crate::world;

fn to_js_value<T: Serialize>(t: &T) -> JsValue { to_value(t).unwrap() }
fn from_js_value<T: DeserializeOwned>(js: JsValue) -> T { 
    from_value(js).unwrap()
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct World {
    wrapped: Rc<RefCell<world::World>>,
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        World {
            wrapped: Rc::new(RefCell::new(world::World::new()))
        }
    }
    #[wasm_bindgen(js_name = clone)]
    pub fn js_clone(&self) -> Self {
        World {
            wrapped: Rc::new(RefCell::new(self.wrapped().clone()))
        }
    }

    // Accessors

    #[wasm_bindgen(getter)]
    pub fn playerId(&self) -> JsValue {
        to_value(&self.wrapped().player_id()).unwrap()
    }

    pub fn getTiles(&self) -> Array /* [Hex, Tile][] */ {
        self.wrapped().map().tiles().iter()
            .map(|(h, t)| {
                let tuple = Array::new();
                tuple.push(&to_js_value::<Hex>(h));
                tuple.push(&to_js_value::<Tile>(t));
                tuple
            })
            .collect()
    }

    pub fn getTile(&self, hex: JsValue) -> JsValue /* Tile | undefined */ {
        self.wrapped().map().tiles()
            .get(&from_js_value::<Hex>(hex))
            .map_or(JsValue::undefined(), |t| to_value(&t).unwrap())
    }

    pub fn getCreatureMap(&self) -> Array /* [Id<Creature>, Hex][] */ {
        self.wrapped().map().creatures().iter()
            .map(|(id, hex)| {
                let tuple = Array::new();
                tuple.push(&to_js_value::<Id<Creature>>(id));
                tuple.push(&to_js_value::<Hex>(hex));
                tuple
            })
            .collect()
    }

    pub fn getCreature(&self, id: JsValue) -> Option<crate::creature::Creature> {
        let id: Id<Creature> = from_js_value(id);
        self.wrapped().creatures().map().get(&id).cloned()
    }

    pub fn getCreatureHex(&self, id: JsValue) -> JsValue /* Hex | undefined */ {
        let id: Id<Creature> = from_js_value(id);
        self.wrapped().map().creatures().get(&id)
            .map_or(JsValue::undefined(), to_js_value::<Hex>)
    }

    pub fn getCreatureRange(&self, id: JsValue) -> Array /* Hex[] */ {
        let id: Id<Creature> = from_js_value(id);
        let wrapped = self.wrapped();
        let range = match wrapped.creatures().map().get(&id) {
            Some(c) => match c.kind() {
                Kind::NPC(npc) => npc.move_range,
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
            .map(JsValue::from)
            .collect()
    }

    pub fn checkSpendAP(&self, creature_id: JsValue, ap: i32) -> bool {
        let id: Id<Creature> = from_js_value(creature_id);
        return self.wrapped().check_action(&Action::SpendAP { id, ap });
    }

    // Mutators

    pub fn npcTurn(&mut self) -> Array /* Event[] */ {
        self.wrapped_mut().npc_turn().into_iter()
            .map(Event::new)
            .map(JsValue::from)
            .collect()
    }

    #[wasm_bindgen(getter)]
    pub fn logging(&self) -> bool { self.wrapped().logging }
    #[wasm_bindgen(setter)]
    pub fn set_logging(&mut self, logging: bool) {
        self.wrapped_mut().logging = logging
    }
}

impl World {
    pub fn wrapped(&self) -> Ref<world::World> { self.wrapped.borrow() }
    pub fn wrapped_mut(&mut self) -> RefMut<world::World> { self.wrapped.borrow_mut() }
}

#[wasm_bindgen]
pub struct Event {
    wrapped: event::Meta<event::Event>,
}

impl Event {
    pub fn new(wrapped: event::Meta<event::Event>) -> Self {
        Event { wrapped }
    }
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl Event {
    #[wasm_bindgen(getter)]
    pub fn tags(&self) -> Array /* string[] */ {
        self.wrapped.tags.iter()
            .map(|s| JsValue::from(s))
            .collect()
    }

    #[wasm_bindgen(getter)]
    pub fn creatureMoved(&self) -> Option<CreatureMoved> {
        CreatureMoved::new(&self.wrapped.data)
    }
}

#[wasm_bindgen]
pub struct CreatureMoved {
    id: JsValue,
    from: JsValue,
    to: JsValue,
}

#[wasm_bindgen]
impl CreatureMoved {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> JsValue { self.id.clone() }
    #[wasm_bindgen(getter)]
    pub fn from(&self) -> JsValue { self.from.clone() }
    #[wasm_bindgen(getter)]
    pub fn to(&self) -> JsValue { self.to.clone() }
}

impl CreatureMoved {
    fn new(ev: &event::Event) -> Option<Self> {
        match ev {
            event::Event::CreatureMoved { id, from, to } => Some(
                CreatureMoved {
                    id: to_js_value::<Id<Creature>>(id),
                    from: to_js_value::<Hex>(from),
                    to: to_js_value::<Hex>(to),
                }
            ),
            _ => None,
        }
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
        self.wrapped.apply(&mut world.wrapped.borrow_mut(), from_js_value::<Hex>(target)).into_iter()
            .map(Event::new)
            .map(JsValue::from)
            .collect()
    }
}