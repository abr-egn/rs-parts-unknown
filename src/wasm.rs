use std::collections::{HashMap, HashSet};
use hex::Hex;
use js_sys::Array;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use serde_wasm_bindgen::{from_value, to_value};
use ts_data_derive::TsData;
use wasm_bindgen::{
    prelude::*,
    JsCast,
};
use crate::{
    card,
    creature,
    event::{Action, Event},
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

    // Accessors

    #[wasm_bindgen(getter, skip_typescript)]
    pub fn playerId(&self) -> JsValue {
        to_js_value(&self.wrapped.player_id())
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn getTile(&self, hex: JsValue) -> JsValue /* Tile | undefined */ {
        self.wrapped.map().tiles()
            .get(&from_js_value::<Hex>(hex))
            .map_or(JsValue::undefined(), |t| to_js_value(&t))
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn getTiles(&self) -> Array /* [Hex, Tile][] */ {
        self.wrapped.map().tiles().iter()
            .map(|(h, t)| {
                let tuple = Array::new();
                tuple.push(&to_js_value::<Hex>(h));
                tuple.push(&to_js_value::<Tile>(t));
                tuple
            })
            .collect()
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn getCreature(&self, id: JsValue) -> JsValue {
        let id: Id<creature::Creature> = from_js_value(id);
        self.wrapped.creatures().map().get(&id)
            .map_or(JsValue::undefined(), |c| Creature::new(id, c).js())
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn getCreatureMap(&self) -> Array /* [Id<Creature>, Hex][] */ {
        self.wrapped.map().creatures().iter()
            .map(|(id, hex)| {
                let tuple = Array::new();
                tuple.push(&to_js_value::<Id<creature::Creature>>(id));
                tuple.push(&to_js_value::<Hex>(hex));
                tuple
            })
            .collect()
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn getCreatureHex(&self, id: JsValue) -> JsValue /* Hex | undefined */ {
        let id: Id<creature::Creature> = from_js_value(id);
        self.wrapped.map().creatures().get(&id)
            .map_or(JsValue::undefined(), to_js_value::<Hex>)
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn getCreatureRange(&self, id: JsValue) -> Array /* Hex[] */ {
        let id: Id<creature::Creature> = from_js_value(id);
        let range = match self.wrapped.creatures().map().get(&id) {
            Some(c) => c.cur_mp,
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

    // TODO(random)
    #[wasm_bindgen(skip_typescript)]
    pub fn checkSpendAP(&self, creature_id: JsValue, ap: i32) -> bool {
        let id: Id<creature::Creature> = from_js_value(creature_id);
        return self.wrapped.check_action(&Action::SpendAP { id, ap });
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn startPlay(&self, card: JsValue) -> Option<Behavior> {
        let card: Card = from_js_value(card);
        let creature = self.wrapped.creatures().map().get(&card.creatureId)?;
        let part = creature.parts.map().get(&card.partId)?;
        let real_card = part.cards.map().get(&card.id)?;
        Some(Behavior::new((real_card.start_play)(&self.wrapped, &card.creatureId)))
    }

    pub fn affectsAction(&self, action: JsValue) -> Array /* string[] */ {
        let action: Action = from_js_value(action);
        let (mods, triggers) = self.wrapped.affects_action(&action);
        let out = Array::new();
        for mod_id in mods {
            let m = match self.wrapped.mods().map().get(&mod_id) {
                Some(m) => m,
                None => continue,
            };
            out.push(&JsValue::from(m.name()));
        }
        for trigger_id in triggers {
            let t = match self.wrapped.triggers().map().get(&trigger_id) {
                Some(t) => t,
                None => continue,
            };
            out.push(&JsValue::from(t.name()));
        }
        out
    }

    // Updates

    #[wasm_bindgen(skip_typescript)]
    pub fn playCard(&self, card: JsValue, behavior: Behavior, target: JsValue) -> Array /* [World, Event[]] */ {
        let card: Card = from_js_value(card);
        let target: Hex = from_js_value(target);
        let mut newWorld = self.wrapped.clone();
        let mut events: Vec<Event> = newWorld.execute(&Action::SpendAP {
            id: card.creatureId,
            ap: card.apCost,
        });
        if !Event::is_failure(&events) {
            events.extend(behavior.wrapped.apply(&mut newWorld, target));
        }
        world_update(newWorld, &events)
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn npcTurn(&self) -> Array /* [World, Event[]] */ {
        let mut newWorld = self.wrapped.clone();
        let events = newWorld.npc_turn();
        world_update(newWorld, &events)
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn movePlayer(&self, to: JsValue) -> Array /* [World, Event[]] */ {
        let to: Hex = from_js_value(to);
        let mut new = self.wrapped.clone();
        let events = new.move_creature(new.player_id(), to);
        world_update(new, &events)
    }

    // Debugging

    #[wasm_bindgen(skip_typescript)]
    pub fn setTracer(&mut self, tracer: JsValue) {
        if tracer.is_undefined() {
            self.wrapped.tracer = None;
        } else {
            self.wrapped.tracer = Some(Box::new(WrapTracer { wrapped: tracer }));
        }
    }
}

fn world_update(new: world::World, events: &[Event]) -> Array {
    let out = Array::new();
    out.push(&JsValue::from(World { wrapped: new }));
    out.push(&JsValue::from(events.iter().map(to_js_value).collect::<Array>()));
    out
}

#[derive(Serialize, TsData)]
#[allow(non_snake_case)]
pub struct Creature {
    id: Id<creature::Creature>,
    parts: HashMap<Id<creature::Part>, Part>,
    curAp: i32,
    curMp: i32,
}

impl Creature {
    fn new(id: Id<creature::Creature>, source: &creature::Creature) -> Creature {
        let parts = source.parts.map().iter()
            .map(|(part_id, part)| (*part_id, Part::new(*part_id, id, part)))
            .collect();
        Creature {
            id,
            parts,
            curAp: source.cur_ap,
            curMp: source.cur_mp,
        }
    }
    fn js(&self) -> JsValue { to_js_value(&self) }
}

#[derive(Serialize, TsData)]
#[allow(non_snake_case)]
pub struct Part {
    id: Id<creature::Part>,
    creatureId: Id<creature::Creature>,
    cards: HashMap<Id<card::Card>, Card>,
    ap: i32,
}

#[allow(non_snake_case)]
impl Part {
    fn new(
        id: Id<creature::Part>,
        creatureId: Id<creature::Creature>,
        source: &creature::Part,
    ) -> Self {
        let cards = source.cards.map().iter()
            .map(|(card_id, card)| (*card_id, Card::new(*card_id, id, creatureId, card)))
            .collect();
        Part {
            id, creatureId, cards,
            ap: source.ap,
        }
    }
}

#[derive(Serialize, Deserialize, TsData)]
#[allow(non_snake_case)]
pub struct Card {
    id: Id<card::Card>,
    partId: Id<creature::Part>,
    creatureId: Id<creature::Creature>,
    name: String,
    apCost: i32,
}

#[allow(non_snake_case)]
impl Card {
    fn new(
        id: Id<card::Card>,
        partId: Id<creature::Part>,
        creatureId: Id<creature::Creature>,
        source: &card::Card,
    ) -> Self {
        Card {
            id, partId, creatureId,
            name: source.name.clone(),
            apCost: source.ap_cost,
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
    #[wasm_bindgen(skip_typescript)]
    pub fn highlight(&self, world: &World, cursor: JsValue) -> Array /* Hex[] */ {
        self.wrapped.highlight(&world.wrapped, from_js_value::<Hex>(cursor)).into_iter()
            .map(|h| to_js_value::<Hex>(&h))
            .collect()
    }
    #[wasm_bindgen(skip_typescript)]
    pub fn targetValid(&self, world: &World, cursor: JsValue) -> bool {
        self.wrapped.target_valid(&world.wrapped, from_js_value::<Hex>(cursor))
    }
    #[wasm_bindgen(skip_typescript)]
    pub fn preview(&self, world: &World, target: JsValue) -> Array /* Action[] */ {
        let target: Hex = from_js_value(target);
        self.wrapped.preview(&world.wrapped, target).iter()
            .map(to_js_value)
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, TsData)]
pub struct Boundary {
    pub hex: Hex,
    pub sides: Vec<hex::Direction>,
}

fn find_boundary(shape: &[Hex]) -> Vec<Boundary> {
    let shape: HashSet<Hex> = shape.into_iter().cloned().collect();
    let mut out = vec![];
    for &hex in &shape {
        let mut bound = Boundary { hex, sides: vec![] };
        for dir in hex::Direction::all() {
            if !shape.contains(&(hex + dir.delta())) {
                bound.sides.push(*dir);
            }
        }
        if !bound.sides.is_empty() {
            out.push(bound);
        }
    }
    out
}

#[allow(unused)]
#[wasm_bindgen(skip_typescript, js_name="findBoundary")]
pub fn js_find_boundary(shape: &Array /* Hex[] */) -> Array /* Boundary[] */ {
    let shape: Vec<Hex> = shape.iter().map(from_js_value).collect();
    find_boundary(&shape).iter().map(to_js_value).collect()
}

#[wasm_bindgen]
extern "C" {
    pub type Tracer;

    #[wasm_bindgen(structural, method)]
    pub fn startAction(this: &Tracer, action: &JsValue);
    #[wasm_bindgen(structural, method)]
    pub fn modAction(this: &Tracer, modName: &str, new: &JsValue);
    #[wasm_bindgen(structural, method)]
    pub fn resolveAction(this: &Tracer, action: &JsValue, event: &JsValue);
}

#[derive(Debug, Clone)]
struct WrapTracer {
    wrapped: JsValue,
}

impl WrapTracer {
    fn wrapped(&self) -> &Tracer {
        self.wrapped.unchecked_ref()
    }
}

impl world::Tracer for WrapTracer {
    fn start_action(&self, action: &Action) {
       self.wrapped().startAction(&to_js_value(action));
    }
    fn mod_action(&self, mod_name: &str, new: &Action) {
        self.wrapped().modAction(mod_name, &to_js_value(new));
    }
    fn resolve_action(&self, action: &Action, event: &Event) {
        self.wrapped().resolveAction(&to_js_value(action), &to_js_value(event));
    }
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND: &'static str = r#"
interface World {
    // Accessors

    readonly playerId: Id<Creature>;
    getTile(hex: Hex): Tile | undefined;
    getTiles(): Array<[Hex, Tile]>;
    getCreature(id: Id<Creature>): Creature | undefined;
    getCreatureMap(): [Id<Creature>, Hex][];
    getCreatureHex(id: Id<Creature>): Hex | undefined;
    getCreatureRange(id: Id<Creature>): Hex[];
    checkSpendAP(id: Id<Creature>, ap: number): boolean;
    startPlay(card: Card): Behavior | undefined;
    affectsAction(action: Action): string[];

    // Updates

    playCard(card: Card, behavior: Behavior, target: Hex): [World, Event[]];
    npcTurn(): [World, Event[]];
    movePlayer(to: Hex): [World, Event[]];

    // Debugging

    setTracer(tracer: Tracer | undefined): void;
}

export interface Hex {
    x: number,
    y: number,
}

export type Id<_> = number;

export interface Tracer {
    startAction: (action: any) => void,
    modAction: (name: string, new_: any) => void,
    resolveAction: (action: any, event: Event) => void,
}

interface Behavior {
    highlight(world: World, cursor: Hex): Hex[];
    targetValid(world: World, cursor: Hex): boolean;
    preview(world: World, target: Hex): Action[];
}

export type Direction = "XY" | "XZ" | "YZ" | "YX" | "ZX" | "ZY";

export function findBoundary(shape: Hex[]): Boundary[];
"#;