use std::sync::mpsc::Sender;
use std::net::TcpListener;

use std::sync::{Mutex, Arc};
use std::thread;

use std::io::Read;

use types::{Command};
use constants::OPERATION_PREFIX;

pub fn tcp_handler(listener: TcpListener, tx_arc_mutex: Arc<Mutex<Sender<Command>>>) {
    for stream in listener.incoming() {
        println!("New connection");

        let tx_arc_mutex = Arc::clone(&tx_arc_mutex);
        thread::spawn(move || {
            let mut stream = stream.unwrap();

            let mut buffer = vec!();
            stream.read_to_end(&mut buffer).unwrap();

            let message = String::from_utf8(buffer).unwrap();
            println!("Got message: {}", message);

            let command = if message.chars().peekable().peek() == Some(&OPERATION_PREFIX) {
                // Handle operation
                parse_operation(&message)
            }
            else {
                let (name, value, timestamp) = handle_reading(&message);
                Some(Command::AddDatapoint(name, value, timestamp))
            };

            if let Some(command) = command {
                tx_arc_mutex.lock().unwrap().send(command).unwrap();
            }
        });
    }
}
fn handle_reading(message: &str) -> (String, f32, Option<f64>) {
    let split = message.split(':').collect::<Vec<_>>();

    let name = split[0].to_string();
    let value = split[1].to_string().parse::<f32>().unwrap();
    let timestamp = split.get(2).and_then(|timestamp_str| {
        match timestamp_str.parse::<f64>() {
            Ok(val) => Some(val),
            Err(e) => {
                println!("Failed to parse {} as a timestamp, ignoring. {}", timestamp_str, e);
                None
            }
        }
    });

    (name, value, timestamp)
}

fn parse_operation(message: &str) -> Option<Command> {
    let without_prefix = message.chars().skip(1).collect::<String>();

    let split = without_prefix.split(':').collect::<Vec<_>>();

    match split[0] {
        "reset" => Some(Command::Reset(split[1].to_string())),
        _ => None
    }
}
