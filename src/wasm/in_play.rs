use js_sys::Array;
use wasm_bindgen::prelude::*;

use crate::{
    action,
    card,
    wasm::{
        world::World,
        from_js_value, to_js_value,
    },
};

#[wasm_bindgen]
pub struct InPlay {
    #[wasm_bindgen(skip)]
    pub wrapped: card::InPlay,
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl InPlay {
    #[wasm_bindgen(skip_typescript)]
    pub fn range(&self, world: &World) -> Array /* Hex[] */ {
        self.wrapped.behavior.range(&world.wrapped).iter()
            .map(to_js_value)
            .collect()
    }
    #[wasm_bindgen(skip_typescript)]
    pub fn targetValid(&self, world: &World, target: JsValue) -> bool {
        let source = action::Path::Part { cid: self.wrapped.creature_id, pid: self.wrapped.part_id };
        self.wrapped.behavior.target_valid(&world.wrapped, &source, &from_js_value::<action::Path>(target))
    }
    #[wasm_bindgen(skip_typescript)]
    pub fn preview(&self, world: &World, target: JsValue) -> Array /* Event[] */ {
        let source = action::Path::Part { cid: self.wrapped.creature_id, pid: self.wrapped.part_id };
        let target: action::Path = from_js_value(target);
        self.wrapped.behavior.preview(&world.wrapped, source, target).iter()
            .map(to_js_value)
            .collect()
    }
    #[wasm_bindgen(getter)]
    pub fn apCost(&self) -> i32 { self.wrapped.ap_cost }
    #[wasm_bindgen(skip_typescript)]
    pub fn getTargetSpec(&self) -> JsValue {
        to_js_value(&self.wrapped.behavior.target_spec())
    }
}

#[wasm_bindgen(typescript_custom_section)]
const INPLAY_TS: &'static str = r#"
interface InPlay {
    range(world: World): Hex[];
    targetValid(world: World, target: Target): boolean;
    preview(world: World, target: Target): Event[];
    getTargetSpec(): TargetSpec;
}
"#;