use hex;
use js_sys::Array;
use wasm_bindgen::prelude::*;

use crate::card;
use crate::creature::{self, Kind};
use crate::event::{self, Action};
use crate::id_map::Id;
use crate::world;

#[wasm_bindgen]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Hex {
    #[wasm_bindgen(readonly)]
    pub x: i32,
    #[wasm_bindgen(readonly)]
    pub y: i32,
}

#[wasm_bindgen]
impl Hex {
    #[wasm_bindgen(constructor)]
    pub fn make(x: i32, y: i32) -> Self {
        Hex { x, y }
    }
}

impl Hex {
    pub fn new(source: hex::Hex) -> Self {
        Hex { x: source.x, y: source.y }
    }
    pub fn old(&self) -> hex::Hex {
        hex::Hex { x: self.x, y: self.y }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct World {
    #[wasm_bindgen(skip)]
    pub wrapped: world::World,
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self { World { wrapped: world::World::new() } }
    #[wasm_bindgen(js_name = clone)]
    pub fn js_clone(&self) -> Self { self.clone() }

    // Accessors

    #[wasm_bindgen(getter)]
    pub fn playerId(&self) -> u32 { self.wrapped.player_id().value() }

    pub fn getTiles(&self) -> Array /* [Hex, Tile][] */ {
        self.wrapped.map().tiles().iter()
            .map(|(h, t)| {
                let tuple = Array::new();
                tuple.push(&JsValue::from(Hex::new(*h)));
                tuple.push(&JsValue::from(t.clone()));
                tuple
            })
            .collect()
    }

    pub fn getTile(&self, hex: &Hex) -> Option<crate::map::Tile> {
        self.wrapped.map().tiles().get(&hex.old()).cloned()
    }

    pub fn getCreatureMap(&self) -> Array /* [Id<Creature>, Hex][] */ {
        self.wrapped.map().creatures().iter()
            .map(|(id, hex)| {
                let tuple = Array::new();
                tuple.push(&JsValue::from(id.value()));
                tuple.push(&JsValue::from(Hex::new(*hex)));
                tuple
            })
            .collect()
    }

    pub fn getCreature(&self, id: u32) -> Option<crate::creature::Creature> {
        self.wrapped.creatures().map().get(&Id::synthesize(id)).cloned()
    }

    pub fn getCreatureHex(&self, id: u32) -> Option<Hex> {
        self.wrapped.map().creatures().get(&Id::synthesize(id))
            .cloned()
            .map(Hex::new)
    }

    pub fn getCreatureRange(&self, id: u32) -> Array /* Hex[] */ {
        let id = Id::synthesize(id);
        let range = match self.wrapped.creatures().map().get(&id) {
            Some(c) => match c.kind() {
                Kind::NPC(npc) => npc.move_range,
                _ => return Array::new(),
            },
            None => return Array::new(),
        };
        let start = match self.wrapped.map().creatures().get(&id) {
            Some(h) => h,
            None => return Array::new(),
        };
        self.wrapped.map().range_from(*start, range).into_iter()
            .map(Hex::new)
            .map(JsValue::from)
            .collect()
    }

    pub fn checkSpendAP(&self, creature_id: u32, ap: i32) -> bool {
        let id: Id<creature::Creature> = Id::synthesize(creature_id);
        return self.wrapped.check_action(&Action::SpendAP { id, ap });
    }

    // Mutators

    pub fn npcTurn(&mut self) -> Array /* Event[] */ {
        self.wrapped.npc_turn().into_iter()
            .map(Event::new)
            .map(JsValue::from)
            .collect()
    }

    #[wasm_bindgen(getter)]
    pub fn logging(&self) -> bool { self.wrapped.logging }
    #[wasm_bindgen(setter)]
    pub fn set_logging(&mut self, logging: bool) { self.wrapped.logging = logging }
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
    pub id: u32,
    pub from: Hex,
    pub to: Hex,
}

impl CreatureMoved {
    fn new(ev: &event::Event) -> Option<Self> {
        match ev {
            event::Event::CreatureMoved { id, from, to } => Some(
                CreatureMoved {
                    id: id.value(),
                    from: Hex::new(*from),
                    to: Hex::new(*to),
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
    pub fn highlight(&self, world: &World, cursor: &Hex) -> Array /* Hex[] */ {
        self.wrapped.highlight(&world.wrapped, cursor.old()).into_iter()
            .map(Hex::new)
            .map(JsValue::from)
            .collect()
    }
    pub fn targetValid(&self, world: &World, cursor: &Hex) -> bool {
        self.wrapped.target_valid(&world.wrapped, cursor.old())
    }
    pub fn apply(&self, world: &mut World, target: &Hex) -> Array /* Event[] */ {
        self.wrapped.apply(&mut world.wrapped, target.old()).into_iter()
            .map(Event::new)
            .map(JsValue::from)
            .collect()
    }
}