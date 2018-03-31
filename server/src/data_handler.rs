use types::{ReadingCollection, Datapoint};
use std::thread;
use std::sync::mpsc::Receiver;
use chrono::{Utc};

pub fn run_data_handler(rx: Receiver<(String, f32, Option<f64>)>, readings: ReadingCollection) {
    thread::spawn(move || {
        loop {
            let (name, value, timestamp) = rx.recv().unwrap();
            let timestamp = timestamp.unwrap_or(Utc::now().timestamp() as f64);

            let mut map = readings.lock().unwrap();
            if !map.contains_key(&name) {
                map.insert(name.clone(), vec!());
            }
            map.get_mut(&name).unwrap().push(Datapoint{timestamp, value});
        }
    });
}
