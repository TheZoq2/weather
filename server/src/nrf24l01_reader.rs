// use std::sync::mpsc::Sender;
// use std::sync::Arc;
// use std::thread::sleep;
// use std::time::Duration;
// 
// use rppal::spi::{Spi, Bus, SlaveSelect, Mode};
// use rppal::gpio::Gpio;
// 
// use embedded_nrf24l01 as nrf;
// use nrf::Configuration;
// use common::nrf::setup_nrf;
// 
// use types::Command;
// 
// fn run_nrf_reader(tx: Arc<Sender<Command>>) {
//     std::thread::spawn(|| {
//         let spi = Spi::new(
//             Bus::Spi0,
//             SlaveSelect::Ss0,
//             nrf::setup::clock_mhz() * 1_000_000,
//             Mode::Mode0,
//         ).expect("Failed to initialise SPI");
// 
//         let gpio = Gpio::new().expect("Failed to get GPIO peripheral");
// 
//         let ce = gpio.get(22).expect("Failed to get ce pin").into_output();
//         let csn = gpio.get(27).expect("Failed to get cs pin").into_output();
// 
//         let mut nrf = nrf::NRF24L01::new(ce, csn, spi)
//             .expect("Failed to initialise nrf");
// 
//         let addr: [u8; 5] = [0x22, 0x22, 0x22, 0x22, 0x22];
// 
//         setup_nrf(&mut nrf, &addr);
//         sleep(Duration::from_millis(10));
// 
//         let mut nrf = nrf
//             .rx()
//             .expect("Failed to go into RX mode");
//         sleep(Duration::from_millis(130));
// 
//         // nrf.set_tx_addr(b"00001").expect("failed to set addres");
// 
// 
//         loop {
//             while let Some(chan) = nrf.can_read().expect("Failed to check for msgs") {
//                 println!("Got a message on channel: {}", chan);
//                 let message = nrf.read().expect("Failed to read");
//                 let decoded = postcard::from_bytes::<common::Message>(&message);
// 
//                 println!("Msg: {:?}", decoded);
//                 // println!("Content: {}", String::from_utf8_lossy(&message));
//             }
//             println!("{:?}", nrf.can_read());
//             sleep(Duration::from_millis(1000));
//         }
//     })
// }
