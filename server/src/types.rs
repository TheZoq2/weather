use std::collections::hash_map::HashMap;
use std::sync::{Mutex, Arc};

pub type ReadingCollection = Arc<Mutex<HashMap<String, Vec<f32>>>>;
