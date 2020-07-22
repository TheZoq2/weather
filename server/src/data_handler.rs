use std::thread;
use std::sync::mpsc::Receiver;

use chrono::{Utc};

use crate::types::{ReadingCollection, Datapoint, Command};

pub fn handle_datapoint((name,value,timestamp): (String, f32, Option<f64>), readings: &ReadingCollection) {
    let timestamp = timestamp.unwrap_or(Utc::now().timestamp() as f64);

    let mut map = readings.lock().unwrap();
    if !map.contains_key(&name) {
        map.insert(name.clone(), vec!());
    }
    map.get_mut(&name).unwrap().push(Datapoint{timestamp, value});
}

pub fn run_command_handler(rx: Receiver<Command>, readings: ReadingCollection) {
    thread::spawn(move || {
        loop {
            let command = rx.recv().unwrap();

            match command {
                Command::Reset(name) => {
                    let mut map = readings.lock().unwrap();
                    if map.contains_key(&name) {
                        map.remove(&name);
                    }
                }
                Command::AddDatapoint(name, value, timestamp) => {
                    handle_datapoint((name, value, timestamp), &readings)
                }
            }
        }
    });
}
