use std::collections::{HashMap, HashSet};
use hex::Hex;
use js_sys::Array;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::{
    prelude::*,
    JsCast,
};
use crate::{
    card,
    creature,
    error::{Error, Result},
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

    #[wasm_bindgen(skip_typescript)]
    pub fn checkSpendAP(&self, creature_id: JsValue, ap: i32) -> bool {
        let id: Id<creature::Creature> = from_js_value(creature_id);
        return self.wrapped.check_action(&Action::SpendAP { id, ap });
    }

    // Mutators

    #[wasm_bindgen(skip_typescript)]
    pub fn startPlay(&self, card: JsValue) -> Option<Behavior> {
        let card: Card = from_js_value(card);
        let creature = self.wrapped.creatures().map().get(&card.creature_id)?;
        let part = creature.parts.map().get(&card.part_id)?;
        let real_card = part.cards.map().get(&card.id)?;
        Some(Behavior::new((real_card.start_play)(&self.wrapped, &card.creature_id)))
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn npcTurn(&mut self) -> Array /* Event[] */ {
        self.wrapped.npc_turn().iter()
            .map(to_js_value)
            .collect()
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn spendAP(&mut self, creature_id: JsValue, ap: i32) -> Array /* Event[] */ {
        let id: Id<creature::Creature> = from_js_value(creature_id);
        self.wrapped.execute(&Action::SpendAP { id, ap })
            .iter()
            .map(to_js_value)
            .collect()
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn movePlayer(&mut self, to: JsValue) -> Array /* Event[] */ {
        self.move_player(to).iter()
            .map(to_js_value)
            .collect()
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn setTracer(&mut self, tracer: JsValue) {
        if tracer.is_undefined() {
            self.wrapped.tracer = None;
        } else {
            self.wrapped.tracer = Some(Box::new(WrapTracer { wrapped: tracer }));
        }
        
    }
}

impl World {
    fn path(&self, to: JsValue) -> Result<Vec<Hex>> {
        let to: Hex = from_js_value(to);
        let from = self.wrapped.map().creatures().get(&self.wrapped.player_id())
            .ok_or(Error::NoSuchCreature)?;
        self.wrapped.map().path_to(*from, to)
    }

    fn move_player(&mut self, to: JsValue) -> Vec<Event> {
        let path = match self.path(to) {
            Ok(p) => p,
            Err(e) => return vec![Event::failed(e)],
        };
        let mut out = vec![];
        let player_id = self.wrapped.player_id();
        for (from, to) in path.iter().zip(path.iter().skip(1)) {
            let actual = match self.wrapped.map().creatures().get(&player_id) {
                Some(h) => h,
                None => {
                    out.push(Event::failed(Error::NoSuchCreature));
                    return out;
                }
            };
            if actual != from && actual.distance_to(*to) > 1 {
                out.push(Event::failed(Error::Obstructed));
                return out;
            }
            let mut mp_evs = self.wrapped.execute(
                &Action::SpendMP { id: player_id, mp: 1 }
            );
            let failed = Event::is_failure(&mp_evs);
            out.append(&mut mp_evs);
            if failed { return out; }
            out.append(&mut self.wrapped.execute(
                &Action::MoveCreature { id: player_id, to: *to }
            ));
        }
        out
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Creature {
    id: Id<creature::Creature>,
    parts: HashMap<Id<creature::Part>, Part>,
    cur_ap: i32,
    cur_mp: i32,
}

impl Creature {
    fn new(id: Id<creature::Creature>, source: &creature::Creature) -> Creature {
        let parts = source.parts.map().iter()
            .map(|(part_id, part)| (*part_id, Part::new(*part_id, id, part)))
            .collect();
        Creature {
            id,
            parts,
            cur_ap: source.cur_ap,
            cur_mp: source.cur_mp,
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
    pub fn apply(&self, world: &mut World, target: JsValue) -> Array /* Event[] */ {
        let target: Hex = from_js_value(target);
        self.wrapped.apply(&mut world.wrapped, target).iter()
            .map(to_js_value)
            .collect()
    }
}

#[derive(Debug, Clone, Serialize)]
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
    pub fn modAction(this: &Tracer, modName: &str, prev: &JsValue, new: &JsValue);
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
    fn mod_action(&self, mod_name: &str, prev: &Action, new: &Action) {
        self.wrapped().modAction(mod_name, &to_js_value(prev), &to_js_value(new));
    }
    fn resolve_action(&self, action: &Action, event: &Event) {
        self.wrapped().resolveAction(&to_js_value(action), &to_js_value(event));
    }
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND: &'static str = r#"
interface World {
    readonly playerId: Id<Creature>;
    getTile(hex: Hex): Tile | undefined;
    getTiles(): Array<[Hex, Tile]>;
    getCreature(id: Id<Creature>): Creature | undefined;
    getCreatureMap(): [Id<Creature>, Hex][];
    getCreatureHex(id: Id<Creature>): Hex | undefined;
    getCreatureRange(id: Id<Creature>): Hex[];
    checkSpendAP(id: Id<Creature>, ap: number): boolean;

    startPlay(card: Card): Behavior | undefined;
    npcTurn(): Event[];
    spendAP(id: Id<Creature>, ap: number): Event[];
    movePlayer(to: Hex): Event[];
    setTracer(tracer: Tracer | undefined): void;
}

export interface Hex {
    x: number,
    y: number,
}

export interface Tile {
    space: Space,
    creature?: number,
}

export type Id<_> = number;

export interface Event {
    Nothing: {} | undefined,
    Failed: {
        action: any,
        reason: string,
    } | undefined,
    CreatureMoved: {
        id: Id<Creature>,
        from: Hex,
        to: Hex,
    } | undefined,
    SpentAP: {
        id: Id<Creature>,
        ap: number,
    } | undefined,
    ChangeMP: {
        id: Id<Creature>,
        mp: number,
    } | undefined,
}

export interface Creature {
    id: Id<Creature>,
    parts: Map<Id<Part>, Part>,
    curAp: number,
    curMp: number,
}

export interface Player {}
export interface NPC {
    move_range: number,
    attack_range: number,
}

export interface Part {
    id: Id<Part>,
    creatureId: Id<Creature>,
    cards: Map<Id<Card>, Card>,
    ap: number,
}

export interface Card {
    id: Id<Card>,
    partId: Id<Part>,
    creatureId: Id<Creature>,
    name: string,
    apCost: number,
}

export interface Tracer {
    startAction: (action: any) => void,
    modAction: (name: string, prev: any, new_: any) => void,
    resolveAction: (action: any, event: Event) => void,
}

interface Behavior {
    highlight(world: World, cursor: Hex): Hex[];
    targetValid(world: World, cursor: Hex): boolean;
    apply(world: World, target: Hex): Event[];
}

export interface Boundary {
    hex: Hex,
    sides: Direction[],
}

export type Direction = "XY" | "XZ" | "YZ" | "YX" | "ZX" | "ZY";

export function findBoundary(shape: Hex[]): Boundary[];
"#;