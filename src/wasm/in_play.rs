use hex::Hex;
use js_sys::Array;
use wasm_bindgen::prelude::*;

use crate::{
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
    pub fn highlight(&self, world: &World, cursor: JsValue) -> Array /* Hex[] */ {
        self.wrapped.behavior.highlight(&world.wrapped, from_js_value::<Hex>(cursor)).into_iter()
            .map(|h| to_js_value::<Hex>(&h))
            .collect()
    }
    #[wasm_bindgen(skip_typescript)]
    pub fn targetValid(&self, world: &World, cursor: JsValue) -> bool {
        self.wrapped.behavior.target_valid(&world.wrapped, from_js_value::<Hex>(cursor))
    }
    #[wasm_bindgen(skip_typescript)]
    pub fn preview(&self, world: &World, target: JsValue) -> Array /* Action[] */ {
        let target: Hex = from_js_value(target);
        self.wrapped.behavior.preview(&world.wrapped, target).iter()
            .map(to_js_value)
            .collect()
    }
    #[wasm_bindgen(getter)]
    pub fn apCost(&self) -> i32 { self.wrapped.ap_cost }
}

#[wasm_bindgen(typescript_custom_section)]
const INPLAY_TS: &'static str = r#"
interface InPlay {
    range(world: World): Hex[];
    highlight(world: World, cursor: Hex): Hex[];
    targetValid(world: World, cursor: Hex): boolean;
    preview(world: World, target: Hex): Action[];
}
"#;