use simple_server::Server;
use http::header;
use serde_json;
use std::thread;

use types::ReadingCollection;

pub fn run_server(listen_address: String, port: u16, readings: ReadingCollection) {
    let server = Server::new(move |request, mut response| {
        let request_path = request.uri().path();
        let request_path_parts = request_path.split('/').collect::<Vec<_>>();

        let request_response = match request_path_parts[1] {
            "data" => {
                let name = request_path_parts.get(2).expect("Data query must specify a data name");

                let readings = readings.lock().unwrap();
                let data = readings.get(*name).expect("No such data");
                serde_json::to_string(&data).unwrap()
            }
            other => format!("unhandled uri: {}", other)
        };

        response.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
        //Ok(response.body(request_response.as_bytes())?)
        Ok(response.body(request_response.into_bytes())?)
    });

    thread::spawn(move || {
        println!("Starting http server: http://localhost:{}", port);
        server.listen(&listen_address, &format!("{}", port));
    });
}
