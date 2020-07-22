use std::time::Duration;
use std::path::{Path, PathBuf};
use std::thread;
use std::collections::hash_map::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::Write;

use serde_json;
use crate::error::Result;
use crate::types::{ReadingCollection, Datapoint};


pub fn run_logger(interval: Duration, file: PathBuf, readings: ReadingCollection) {
    thread::spawn(move || {
        loop {
            thread::sleep(interval);
            {
                info!("Saving data");
                let readings = readings.lock().unwrap();

                if let Err(e) = save_data(&file, &*readings) {
                    error!("Failed to log data {:?}", e);
                };

                info!("Data saved");
            }
        }
    });
}

fn save_data(filename: &Path, readings: &HashMap<String, Vec<Datapoint>>) -> Result<()>{
    let saved_string = serde_json::to_string(readings)?;

    let mut file = OpenOptions::new().write(true).create(true).open(filename)?;
    file.write_all(saved_string.as_bytes())?;

    Ok(())
}

pub fn load_data(filename: &Path) -> Result<HashMap<String, Vec<Datapoint>>> {
    let mut file = File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(serde_json::from_str(&content)?)
}

