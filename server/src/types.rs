use std::collections::hash_map::HashMap;
use std::sync::{Mutex, Arc};

#[derive(Serialize)]
pub struct Datapoint {
    pub timestamp: i64,
    pub value: f32
}

pub type ReadingCollection = Arc<Mutex<HashMap<String, Vec<Datapoint>>>>;
