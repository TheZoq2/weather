#![feature(plugin)]
#![plugin(clippy)]

use std::io::prelude::*;
use std::net::{TcpListener};

use std::thread;
use std::sync::mpsc::{channel};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

extern crate serde_json;
#[macro_use]
extern crate serde_derive;

extern crate simple_server;
extern crate http;
extern crate chrono;

mod web;
mod data_handler;
mod types;
mod dummy_data;

fn main() {
    // Setting up thread-shared data
    let reading_collection = Arc::new(Mutex::new(HashMap::new()));
    let (tx, rx) = channel();

    dummy_data::sin_providier(tx.clone(), "temperature".into(), 20., 10.);
    web::run_server(Arc::clone(&reading_collection));
    data_handler::run_data_handler(rx, Arc::clone(&reading_collection));
    let listener = TcpListener::bind("0.0.0.0:2000").unwrap();

    println!("Listener started, waiting for connections on port 2000");

    let tx_arc_mutex = Arc::new(Mutex::new(tx));

    for stream in listener.incoming() {
        println!("New connection");

        let tx_arc_mutex = Arc::clone(&tx_arc_mutex);
        thread::spawn(move || {
            let mut stream = stream.unwrap();

            let mut buffer = vec!();
            stream.read_to_end(&mut buffer).unwrap();

            let message = String::from_utf8(buffer).unwrap();
            println!("Got message: {}", message);
            let split = message.split(':').collect::<Vec<_>>();

            let name = split[0].to_string();
            let value = split[1].to_string().parse::<i32>().unwrap() as f32 / 100.;
            tx_arc_mutex.lock().unwrap().send((name, value)).unwrap();
        });
    }
}
