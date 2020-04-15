use hex::Hex;
use js_sys::Array;
use wasm_bindgen::prelude::*;
use crate::{
    card,
    wasm::{
        World,
        from_js_value, to_js_value,
    },
};

#[wasm_bindgen]
#[derive(Clone)]
pub struct Behavior {
    #[wasm_bindgen(skip)]
    pub wrapped: Box<dyn card::Behavior>,
}

impl Behavior {
    pub fn new(wrapped: Box<dyn card::Behavior>) -> Self {
        Behavior { wrapped }
    }
}

#[allow(non_snake_case)]
#[wasm_bindgen]
impl Behavior {
    #[wasm_bindgen(skip_typescript)]
    pub fn range(&self, world: &World) -> Array /* Hex[] */ {
        self.wrapped.range(&world.wrapped).iter()
            .map(to_js_value)
            .collect()
    }
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

#[wasm_bindgen(typescript_custom_section)]
const BEHAVIOR_TS: &'static str = r#"
interface Behavior {
    range(world: World): Hex[];
    highlight(world: World, cursor: Hex): Hex[];
    targetValid(world: World, cursor: Hex): boolean;
    preview(world: World, target: Hex): Action[];
}
"#;