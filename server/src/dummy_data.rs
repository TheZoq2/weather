use std::sync::mpsc::Sender;
use std::thread;
use std::f32;

use crate::types::Command;

pub fn sin_provider(tx: Sender<Command>, name: String, amplitude: f32, bias: f32) {
    thread::spawn(move || {
        let mut t: f32 = 0.;
        loop {
            tx.send(
                Command::AddDatapoint(name.clone(), t.sin() * amplitude + bias, None)
            ).unwrap();

            t += 0.3;
            thread::sleep(::std::time::Duration::new(10,0));
        }
    });
}
