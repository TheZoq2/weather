use std::thread::sleep_ms;

use rppal::spi::{Spi, Bus, SlaveSelect, Mode};
use rppal::gpio::Gpio;

use embedded_nrf24l01 as nrf;
use nrf::Configuration;

fn main() {
    let spi = Spi::new(
        Bus::Spi0,
        SlaveSelect::Ss0,
        nrf::setup::clock_mhz() * 1_000_000,
        Mode::Mode0,
    ).expect("Failed to initialise SPI");

    let gpio = Gpio::new().expect("Failed to get GPIO peripheral");

    let ce = gpio.get(22).expect("Failed to get ce pin").into_output();
    let csn = gpio.get(27).expect("Failed to get cs pin").into_output();

    let mut nrf = nrf::NRF24L01::new(ce, csn, spi)
        .expect("Failed to initialise nrf");

    let addr: [u8; 5] = [0x22, 0x22, 0x22, 0x22, 0x22];

    nrf.set_frequency(100).unwrap();
    nrf.set_auto_retransmit(0, 0).unwrap();
    // nrf.set_crc(Some(CrcMode::TwoBytes)).unwrap();
    // nrf.set_rf(DataRate::R250Kbps, 1).unwrap();
    nrf
        .set_auto_ack(&[true, false, false, false, false, false])
        .unwrap();
    nrf
        .set_pipes_rx_enable(&[true, false, false, false, false, false])
        .unwrap();
    nrf
        .set_pipes_rx_lengths(&[None, None, None, None, None, None])
        .unwrap();
    nrf.set_tx_addr(&addr).unwrap();
    nrf.set_rx_addr(0, &addr).unwrap();
    nrf.flush_rx().unwrap();
    nrf.flush_tx().unwrap();

    sleep_ms(10);

    let mut nrf = nrf
        .rx()
        .expect("Failed to go into RX mode");

    println!("AutoAck {:?}", nrf.get_auto_ack().unwrap());
    println!("Register {:?}", nrf.get_address_width().unwrap());
    println!("Frequency {:?}", nrf.get_frequency().unwrap());

    sleep_ms(130);

    // nrf.set_tx_addr(b"00001").expect("failed to set addres");


    loop {
        if let Some(chan) = nrf.can_read().expect("Failed to check for msgs") {
            println!("Got a message on channel: {}", chan);
            let message = nrf.read().expect("Failed to read");
            println!("Content: {}", String::from_utf8_lossy(&message));
        }
        println!("{:?}", nrf.can_read());
        sleep_ms(1000);
    }
    println!("Hello, world!");
}
