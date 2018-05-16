use std::collections::hash_map::HashMap;
use std::sync::{Mutex, Arc};

#[derive(Serialize, Deserialize)]
pub struct Datapoint {
    pub timestamp: f64,
    pub value: f32
}

pub enum Command {
    Reset(String), // Removes all data for the specified reading
    AddDatapoint(String, f32, Option<f64>)
}

pub type ReadingCollection = Arc<Mutex<HashMap<String, Vec<Datapoint>>>>;



