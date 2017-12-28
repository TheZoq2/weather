use types::ReadingCollection;
use std::thread;
use std::sync::mpsc::Receiver;

pub fn run_data_handler(rx: Receiver<(String, f32)>, readings: ReadingCollection) {
    thread::spawn(move || {
        loop {
            let (name, value) = rx.recv().unwrap();
            println!("Data handler got {} : {}", name, value);

            let mut map = readings.lock().unwrap();
            if !map.contains_key(&name) {
                map.insert(name.clone(), vec!());
            }
            map.get_mut(&name).unwrap().push(value);
        }
    });
}
