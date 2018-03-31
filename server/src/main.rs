#![feature(plugin)]
//#![plugin(clippy)]

use std::io::prelude::*;
use std::net::{TcpListener};

use std::thread;
use std::sync::mpsc::{channel};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

extern crate serde_json;
extern crate toml;
#[macro_use]
extern crate serde_derive;

extern crate simple_server;
extern crate http;
extern crate chrono;
#[macro_use]
extern crate error_chain;

mod web;
mod data_handler;
mod types;
mod dummy_data;
mod logger;
mod error;
mod config;

fn main() {
    let config = config::read_config(&PathBuf::from("config.toml")).unwrap();

    let reading_collection = Arc::new(Mutex::new(
        logger::load_data(&config.log_filename).unwrap_or_else(|_| HashMap::new())
    ));

    //let reading_collection = Arc::new(Mutex::new(HashMap::new()));
    let (tx, rx) = channel();

    logger::run_logger(
            Duration::from_secs(60),
            config.log_filename,
            Arc::clone(&reading_collection)
        );
    // dummy_data::sin_provider(
    //         tx.clone(),
    //         "temperature".into(),
    //         20.,
    //         10.
    //     );
    web::run_server(
            config.http_address.clone(),
            config.http_port,
            Arc::clone(&reading_collection)
        );
    data_handler::run_data_handler(
            rx,
            Arc::clone(&reading_collection)
        );


    let listener = TcpListener::bind(&format!("{}:{}", config.tcp_address, config.tcp_port)).unwrap();

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

            tx_arc_mutex.lock().unwrap().send((name, value, timestamp)).unwrap();
        });
    }
}
