use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use hex::Hex;
use js_sys::Array;
use log::info;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

use crate::card;
use crate::creature::{self, Kind};
use crate::event::{self, Action};
use crate::id_map::Id;
use crate::world;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct World {
    wrapped: Rc<RefCell<world::World>>,
}

impl Drop for World {
    fn drop(&mut self) {
        info!("World dropped");
    }
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
    pub fn playerId(&self) -> u32 { self.wrapped().player_id().value() }

    pub fn getTiles(&self) -> Array /* [Hex, Tile][] */ {
        self.wrapped().map().tiles().iter()
            .map(|(h, t)| {
                let tuple = Array::new();
                tuple.push(&to_js_hex(h));
                tuple.push(&JsValue::from(t.clone()));
                tuple
            })
            .collect()
    }

    pub fn getTile(&self, hex: JsValue) -> Option<crate::map::Tile> {
        self.wrapped().map().tiles().get(&from_js_hex(hex)).cloned()
    }

    pub fn getCreatureMap(&self) -> Array /* [Id<Creature>, Hex][] */ {
        self.wrapped().map().creatures().iter()
            .map(|(id, hex)| {
                let tuple = Array::new();
                tuple.push(&JsValue::from(id.value()));
                tuple.push(&to_js_hex(hex));
                tuple
            })
            .collect()
    }

    pub fn getCreature(&self, id: u32) -> Option<crate::creature::Creature> {
        self.wrapped().creatures().map().get(&Id::synthesize(id)).cloned()
    }

    pub fn getCreatureHex(&self, id: u32) -> JsValue /* Hex | undefined */ {
        self.wrapped().map().creatures().get(&Id::synthesize(id))
            .cloned()
            .map_or(JsValue::undefined(), |h| to_js_hex(&h))
    }

    pub fn getCreatureRange(&self, id: u32) -> Array /* Hex[] */ {
        let id = Id::synthesize(id);
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
            .map(|hex| to_js_hex(&hex))
            .map(JsValue::from)
            .collect()
    }

    pub fn checkSpendAP(&self, creature_id: u32, ap: i32) -> bool {
        let id: Id<creature::Creature> = Id::synthesize(creature_id);
        return self.wrapped().check_action(&Action::SpendAP { id, ap });
    }

    #[wasm_bindgen(getter)]
    pub fn refCount(&self) -> usize { Rc::strong_count(&self.wrapped) }

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

fn to_js_hex(hex: &Hex) -> JsValue { to_value(hex).unwrap() }

fn from_js_hex(js: JsValue) -> Hex { from_value(js).unwrap() }

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
    #[wasm_bindgen(readonly)]
    pub id: u32,
    from: JsValue,
    to: JsValue,
}

#[wasm_bindgen]
impl CreatureMoved {
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
                    id: id.value(),
                    from: to_js_hex(from),
                    to: to_js_hex(to),
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
        self.wrapped.highlight(&world.wrapped.borrow(), from_js_hex(cursor)).into_iter()
            .map(|h| to_js_hex(&h))
            .collect()
    }
    pub fn targetValid(&self, world: &World, cursor: JsValue) -> bool {
        self.wrapped.target_valid(&world.wrapped.borrow(), from_js_hex(cursor))
    }
    pub fn apply(&self, world: &mut World, target: JsValue) -> Array /* Event[] */ {
        self.wrapped.apply(&mut world.wrapped.borrow_mut(), from_js_hex(target)).into_iter()
            .map(Event::new)
            .map(JsValue::from)
            .collect()
    }
}