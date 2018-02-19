use std::sync::mpsc::Sender;
use std::thread;

use std::f32;

pub fn sin_provider(tx: Sender<(String, f32)>, name: String, amplitude: f32, bias: f32) {
    thread::spawn(move || {
        let mut t: f32 = 0.;
        loop {
            tx.send((name.clone(), t.sin() * amplitude + bias)).unwrap();

            t += 0.3;
            thread::sleep(::std::time::Duration::new(10,0));
        }
    });
}
