use std::time::Duration;
use std::path::{Path, PathBuf};
use std::thread;
use std::collections::hash_map::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

use serde_json;
use error::Result;
use types::{ReadingCollection, Datapoint};


pub fn run_logger(interval: Duration, file: PathBuf, readings: ReadingCollection) {
    thread::spawn(move || {
        println!("Saving data");
        let readings = readings.lock().unwrap();

        if let Err(e) = save_data(&file, &*readings) {
            println!("Failed to log data {:?}", e);
        };

        println!("Data saved");

        thread::sleep(interval);
    });
}

fn save_data(filename: &Path, readings: &HashMap<String, Vec<Datapoint>>) -> Result<()>{
    let saved_string = serde_json::to_string(readings)?;

    let mut file = OpenOptions::new().write(true).create(true).open(filename)?;
    file.write_all(saved_string.as_bytes())?;

    Ok(())
}

