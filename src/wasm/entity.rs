use std::collections::HashMap;

use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    //id_map::Id,
    entity,
    status::{self, StatusId},
};

#[derive(Serialize, TsData)]
#[allow(non_snake_case)]
pub struct Entity {
    status: HashMap<StatusId, Status>,
}

impl Entity {
    pub fn new(source: &entity::Entity) -> Self {
        let status = source.status.iter()
            .map(|(id, status)| (*id, Status::new(&**status)))
            .collect();
        Entity { status }
    }
}

#[derive(Serialize, TsData)]
#[allow(non_snake_case)]
pub struct Status {
    name: String,
}

impl Status {
    fn new<S: status::Status + ?Sized>(source: &S) -> Self {
        Status {
            name: source.name().into(),
        }
    }
}