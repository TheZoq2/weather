use std::io::prelude::*;
use std::net::{TcpListener};

use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::io;
use std::collections::HashMap;

extern crate serde_json;
extern crate simple_server;
use simple_server::Server;
extern crate http;
use http::header;

type ReadingCollection = Arc<Mutex<HashMap<String, Vec<f32>>>>;

fn run_server(readings: ReadingCollection) {
    let host = "0.0.0.0";
    let port = "8080";

    let server = Server::new(move |request, mut response| {

        let request_path = request.uri().path();
        let request_path_parts = request_path.split("/").collect::<Vec<_>>();

        println!("got request: {:?} {}", request_path_parts, request.uri());

        let request_response = match request_path_parts[1] {
            "data" => {
                let name = request_path_parts.get(2).expect("Data query must specify a data name");

                let readings = readings.lock().unwrap();
                let data = readings.get(*name).expect("No such data");
                serde_json::to_string(&data).unwrap()
            }
            other => String::from("unhandled uri: {}")
        };

        response.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".as_bytes());
        //Ok(response.body(request_response.as_bytes())?)
        Ok(response.body(request_response.into_bytes())?)
    });

    thread::spawn(move || {
        println!("Starting http server: http://localhost:{}", port);
        server.listen(host, port);
    });
}

fn run_data_handler(rx: Receiver<(String, f32)>, readings: ReadingCollection) {
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

fn main() {
    // Setting up thread-shared data
    let reading_collection = Arc::new(Mutex::new(HashMap::new()));
    let (tx, rx) = channel();

    run_server(reading_collection.clone());
    run_data_handler(rx, reading_collection.clone());
    let listener = TcpListener::bind("0.0.0.0:2000").unwrap();

    println!("Listener started, waiting for connections on port 2000");

    let tx_arc_mutex = Arc::new(Mutex::new(tx));

    for stream in listener.incoming() {
        println!("New connection");

        let tx_arc_mutex = tx_arc_mutex.clone();
        thread::spawn(move || {
            let mut stream = stream.unwrap();

            let mut buffer = vec!();
            stream.read_to_end(&mut buffer).unwrap();

            let message = String::from_utf8(buffer).unwrap();
            println!("Got message: {}", message);
            let split = message.split(":").collect::<Vec<_>>();

            let name = split[0].to_string();
            let value = split[1].to_string().parse::<i32>().unwrap() as f32 / 100.;
            tx_arc_mutex.lock().unwrap().send((name, value));
        });
    }
}
