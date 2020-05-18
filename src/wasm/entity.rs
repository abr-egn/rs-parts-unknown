use std::collections::HashMap;

use serde::{Serialize};
use ts_data_derive::TsData;
use wasm_bindgen::prelude::*;

use crate::{
    //id_map::Id,
    status::{/*self,*/ StatusId},
};

#[derive(Serialize, TsData)]
#[allow(non_snake_case)]
pub struct Entity {
    status: HashMap<StatusId, Status>,
}

#[derive(Serialize, TsData)]
#[allow(non_snake_case)]
pub struct Status {
    name: String,
}