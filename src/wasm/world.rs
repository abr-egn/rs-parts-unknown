use hex::Hex;
use js_sys::Array;
use wasm_bindgen::{
    prelude::*,
    JsCast,
};

use crate::{
    creature::{self, CreatureAction},
    event::{Action, Event},
    id_map::Id,
    map::Tile,
    wasm::{
        behavior::Behavior,
        card::Card,
        creature::Creature,
        from_js_value, to_js_value,
    },
    world,
};

#[wasm_bindgen]
#[derive(Debug)]
pub struct World {
    #[wasm_bindgen(skip)]
    pub wrapped: world::World,
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
        self.wrapped.creatures().get(id)
            .map_or(JsValue::undefined(), |c| Creature::new(id, c).js())
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn getCreatureIds(&self) -> Array {
        self.wrapped.creatures().keys()
            .map(to_js_value)
            .collect()
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
        let range = match self.wrapped.creatures().get(id) {
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
        match self.wrapped.creatures().get(id) {
            Some(c) => c.cur_ap >= ap,
            None => false,
        }
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn startPlay(&self, card: JsValue) -> Option<Behavior> {
        let card: Card = from_js_value(card);
        let creature = self.wrapped.creatures().get(card.creatureId)?;
        let part = creature.parts.get(card.partId)?;
        let real_card = part.cards.get(card.id)?;
        Some(Behavior::new((real_card.start_play)(&self.wrapped, &card.creatureId)))
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn affectsAction(&self, action: JsValue) -> Array /* string[] */ {
        let action: Action = from_js_value(action);
        let (mods, triggers) = self.wrapped.affects_action(&action);
        let out = Array::new();
        for mod_id in mods {
            let m = match self.wrapped.mods().get(mod_id) {
                Some(m) => m,
                None => continue,
            };
            out.push(&JsValue::from(m.name()));
        }
        for trigger_id in triggers {
            let t = match self.wrapped.triggers().get(trigger_id) {
                Some(t) => t,
                None => continue,
            };
            out.push(&JsValue::from(t.name()));
        }
        out
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn path(&self, from: JsValue, to: JsValue) -> Array /* Hex[] */ {
        let from: Hex = from_js_value(from);
        let to: Hex = from_js_value(to);
        self.wrapped.map().path_to(from, to).unwrap_or(vec![]).iter()
            .map(to_js_value)
            .collect()
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn state(&self) -> JsValue /* GameState */ {
        to_js_value(&self.wrapped.state())
    }

    // Updates

    #[wasm_bindgen(skip_typescript)]
    pub fn playCard(&self, card: JsValue, behavior: Behavior, target: JsValue) -> Array /* [World, Event[]] */ {
        let card: Card = from_js_value(card);
        let target: Hex = from_js_value(target);
        let mut newWorld = self.wrapped.clone();
        let mut events: Vec<Event> = newWorld.execute(&Action::ToCreature {
            id: card.creatureId,
            action: CreatureAction::SpendAP { ap: card.apCost },
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

#[wasm_bindgen(typescript_custom_section)]
const WORLD_TS: &'static str = r#"
interface World {
    // Accessors

    readonly playerId: Id<Creature>;
    getTile(hex: Hex): Tile | undefined;
    getTiles(): Array<[Hex, Tile]>;
    getCreature(id: Id<Creature>): Creature | undefined;
    getCreatureIds(): Id<Creature>[];
    getCreatureMap(): [Id<Creature>, Hex][];
    getCreatureHex(id: Id<Creature>): Hex | undefined;
    getCreatureRange(id: Id<Creature>): Hex[];
    checkSpendAP(id: Id<Creature>, ap: number): boolean;
    startPlay(card: Card): Behavior | undefined;
    affectsAction(action: Action): string[];
    path(from: Hex, to: Hex): Hex[];
    state(): GameState;

    // Updates

    playCard(card: Card, behavior: Behavior, target: Hex): [World, Event[]];
    npcTurn(): [World, Event[]];
    movePlayer(to: Hex): [World, Event[]];

    // Debugging

    setTracer(tracer: Tracer | undefined): void;
}
"#;

fn world_update(new: world::World, events: &[Event]) -> Array {
    let out = Array::new();
    out.push(&JsValue::from(World { wrapped: new }));
    out.push(&JsValue::from(events.iter().map(to_js_value).collect::<Array>()));
    out
}

#[wasm_bindgen]
extern "C" {
    pub type Tracer;

    #[wasm_bindgen(structural, method)]
    pub fn startAction(this: &Tracer, action: &JsValue);
    #[wasm_bindgen(structural, method)]
    pub fn modAction(this: &Tracer, modName: &str, new: &JsValue);
    #[wasm_bindgen(structural, method)]
    pub fn resolveAction(this: &Tracer, action: &JsValue, events: &Array);
}

#[wasm_bindgen(typescript_custom_section)]
const TRACER_TS: &'static str = r#"
export interface Tracer {
    startAction: (action: Action) => void,
    modAction: (name: string, new_: Action) => void,
    resolveAction: (action: Action, events: [Event]) => void,
}
"#;

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
    fn resolve_action(&self, action: &Action, events: &[Event]) {
        let events: Array = events.iter().map(to_js_value).collect();
        self.wrapped().resolveAction(&to_js_value(action), &events);
    }
}