use simple_server::Server;
use http::{header, StatusCode};
use serde_json;
use std::thread;

use std::fs::File;
use std::io::prelude::*;

use types::ReadingCollection;

use error::{Result, ErrorKind};

fn handle_data_request_query(
    request_path_parts: &[&str],
    readings: &ReadingCollection
) -> Result<String> {
    // If a datafield is specified, return that data
    if let Some(name) = request_path_parts.get(2) {
        let readings = readings.lock().unwrap();
        let data = readings.get(*name)
            .ok_or(ErrorKind::NoSuchDataName(name.to_string()));

        Ok(serde_json::to_string(&data?)?)
    }
    // Otherwise return a list of available data
    else {
        let readings = readings.lock().unwrap();
        let available_data = readings.keys().collect::<Vec<_>>();
        Ok(serde_json::to_string(&available_data)?)
    }
}

fn handle_index_request() -> Result<String> {
    let mut file = File::open("frontend/output/index.html")?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn run_server(listen_address: String, port: u16, readings: ReadingCollection) {
    let server = Server::new(move |request, mut response| {
        let request_path = request.uri().path();
        let request_path_parts = request_path.split('/').collect::<Vec<_>>();

        let (handled, content_type) = match request_path_parts[1] {
            "" => {
                (handle_index_request(), "text/html")
            }
            "data" => {
                (handle_data_request_query(&request_path_parts, &readings), "text/plain")
            }
            _ => (Err(ErrorKind::UnhandledURI(request_path.to_string()).into()), "text/plain")
        };

        let request_response = match handled {
            Ok(val) => val,
            Err(e) => {
                response.status(StatusCode::NOT_FOUND);
                format!("{}", e)
            }
        };

        response.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
        response.header(header::CONTENT_TYPE, content_type);
        //Ok(response.body(request_response.as_bytes())?)
        Ok(response.body(request_response.into_bytes())?)
    });

    thread::spawn(move || {
        info!("Starting http server: http://localhost:{}", port);
        server.listen(&listen_address, &format!("{}", port));
    });
}
