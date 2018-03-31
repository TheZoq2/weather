use std::collections::hash_map::HashMap;
use std::sync::{Mutex, Arc};

#[derive(Serialize, Deserialize)]
pub struct Datapoint {
    pub timestamp: f64,
    pub value: f32
}

pub type ReadingCollection = Arc<Mutex<HashMap<String, Vec<Datapoint>>>>;
