use hex::Hex;
use js_sys::Array;
use wasm_bindgen::{
    prelude::*,
    JsCast,
};

use crate::{
    action::{Action, Event, Path},
    card,
    creature,
    id_map::Id,
    map::{Space, Tile},
    npc,
    part::{PartTag},
    wasm::{
        card::Card,
        creature::Creature,
        in_play::InPlay,
        from_js_value, to_js_value,
    },
    world,
    world_ext::WorldExt,
    some_or,
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
    pub fn getCreatures(&self) -> Array {
        self.wrapped.creatures().iter()
            .map(|(id, c)| Creature::new(*id, c).js())
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
        let space_only = id != self.wrapped.player_id();
        self.wrapped.map().range_from(*start, range, space_only).into_iter()
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
    pub fn isPlayable(&self, card: JsValue) -> bool {
        let card: Card = from_js_value(card);
        let creature = some_or!(self.wrapped.creatures().get(card.creatureId), return false);
        if creature.cur_ap < card.apCost { return false; }
        let part = some_or!(creature.parts.get(card.partId), return false);
        return !part.tags().contains(&PartTag::Broken);
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn startPlay(&self, creature_id: JsValue, hand_ix: JsValue) -> Option<InPlay> {
        let creature_id: Id<creature::Creature> = from_js_value(creature_id);
        let hand_ix: usize = from_js_value(hand_ix);
        card::Card::start_play(&self.wrapped, creature_id, hand_ix).ok()
            .map(|ip| InPlay { wrapped: ip })
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

    #[wasm_bindgen(skip_typescript)]
    pub fn simulateMove(&self, to: JsValue) -> Array /* Event[] */ {
        let to: Hex = from_js_value(to);
        let mut new = self.wrapped.clone();
        new.tracer = None;
        new.move_creature(new.player_id(), to).iter()
            .map(to_js_value).collect()
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn shadeFrom(&self, hex: JsValue, id: JsValue) -> Array /* Hex[] */ {
        let hex: Hex = from_js_value(hex);
        let id: Id<creature::Creature> = from_js_value(id);
        let los = self.wrapped.map().los_from(hex, id);
        self.wrapped.map().tiles().iter()
            .filter_map(|(h, t)|
                if t.space == Space::Empty && !los.contains(h) { Some(h) }
                else { None }
            )
            .map(to_js_value)
            .collect()
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn scaledIntent(&self, cid: JsValue) -> JsValue {
        let cid: Id<creature::Creature> = from_js_value(cid);
        match self.scaled_intent(cid) {
            None => JsValue::undefined(),
            Some(intent) => to_js_value(&intent),
        }
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn cardUI(&self, js_card: JsValue, target: JsValue) -> JsValue {
        let js_card: Card = from_js_value(js_card);
        let target: Path = from_js_value(target);
        let card = js_card.get(&self.wrapped).unwrap();
        let ui = (card.ui)(&self.wrapped, &js_card.source(), &target);
        JsValue::from_serde(&ui).unwrap()
    }

    // Updates

    #[wasm_bindgen(skip_typescript)]
    pub fn finishPlay(&self, in_play: InPlay, target: JsValue) -> Array /* [World, Event[]] */ {
        let target: Path = from_js_value(target);
        let mut newWorld = self.wrapped.clone();
        let events = in_play.wrapped.finish(&mut newWorld, &target);
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

impl World {
    fn scaled_intent(&self, cid: Id<creature::Creature>) -> Option<npc::Intent> {
        let creature = self.wrapped.creatures().get(cid)?;
        let npc = creature.npc.as_ref()?;
        let mut intent = npc.intent.clone();
        match &mut intent.kind {
            npc::IntentKind::Stunned => Some(intent),
            npc::IntentKind::Attack { damage, .. } => {
                let source = match npc.intent.from {
                    None => Path::Creature { cid },
                    Some(pid) => Path::Part { cid, pid },
                };
                let target = Path::Part { cid: self.wrapped.player_id(), pid: Id::invalid() };
                let scopes = vec![world::Scope::SourcePart, world::Scope::SourceCreature, world::Scope::World];
                match self.wrapped.scale_damage(&source, &target, *damage, scopes).0 {
                    Some(new_damage) => {
                        *damage = new_damage;
                        Some(intent)
                    }
                    None => None,
                }
            }
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
    getCreatures(): Creature[];
    getCreatureMap(): [Id<Creature>, Hex][];
    getCreatureHex(id: Id<Creature>): Hex | undefined;
    getCreatureRange(id: Id<Creature>): Hex[];
    isPlayable(card: Card): boolean;
    startPlay(creatureId: Id<Creature>, handIx: number): InPlay | undefined;
    path(from: Hex, to: Hex): Hex[];
    state(): GameState;
    simulateMove(to: Hex): Event[];
    shadeFrom(hex: Hex, id: Id<Creature>): Hex[];
    scaledIntent(cid: Id<Creature>): Intent | undefined;
    cardUI(card: Card, target: Path): any;

    // Updates

    finishPlay(inPlay: InPlay, target: Path): [World, Event[]];
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
    pub fn resolveAction(this: &Tracer, action: &JsValue, events: &Array);
    #[wasm_bindgen(structural, method)]
    pub fn systemEvent(this: &Tracer, events: &Array);
}

#[wasm_bindgen(typescript_custom_section)]
const TRACER_TS: &'static str = r#"
export interface Tracer {
    resolveAction: (action: string, events: Event[]) => void,
    systemEvent: (events: Event[]) => void,
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
    fn resolve_action(&self, action: &Action, events: &[Event]) {
        let events: Array = events.iter().map(to_js_value).collect();
        let action = format!("{:?}", action);
        self.wrapped().resolveAction(&to_js_value(&action), &events);
    }
    fn system_event(&self, events: &[Event]) {
        let events: Array = events.iter().map(to_js_value).collect();
        self.wrapped().systemEvent(&events);
    }
}
