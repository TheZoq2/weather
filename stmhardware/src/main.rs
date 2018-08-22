
#![feature(proc_macro)]
#![no_main]
#![no_std]
#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(generic_associated_types)]
#![feature(start)]
#![feature(never_type)]

// extern crate f3;
#[macro_use(block)]
extern crate nb;


// #[macro_use]
extern crate cortex_m;
extern crate cortex_m_semihosting;

#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;

extern crate embedded_hal as hal;
extern crate embedded_hal_time;
//extern crate stm32f30x_hal;
extern crate stm32f103xx_hal;
extern crate stm32f103xx;
extern crate arrayvec;
extern crate panic_semihosting;
extern crate itoa;
extern crate dhtxx;

use cortex_m::asm;
use rt::ExceptionFrame;

// For stdout
use cortex_m_semihosting::hio;
use core::fmt::Write;

use stm32f103xx_hal::prelude::*;
use stm32f103xx_hal::serial::{Serial};
//use stm32f103xx_hal::stm32f103xx::{self};
use stm32f103xx_hal::time::{Hertz, MonoTimer};
use stm32f103xx_hal::timer::Timer;
use stm32f103xx_hal::gpio::gpioa::{CRL};
use stm32f103xx_hal::gpio::gpiob;
use stm32f103xx::TIM4;
use embedded_hal_time::{RealCountDown, Microsecond, Second, Millisecond};

mod serial;
mod esp8266;
mod anemometer;
mod communication;
mod api;
mod types;
mod error;

type ErrorString = arrayvec::ArrayString<[u8; 128]>;

// const IP_ADDRESS: &str = "46.59.41.53";
const IP_ADDRESS: &str = "192.168.0.11";
// const READ_INTERVAL: Second = Second(60*5);
// const READ_INTERVAL: Second = Second(30);
// Sleep for 5 minutes between reads, but wake up once every 10 seconds seconds
// to keep power banks from shutting down the psu
const WAKEUP_INTERVAL: Second = Second(10);
const SLEEP_ITERATIONS: u8 = 6;

macro_rules! handle_result {
    ($result:expr, $storage:ident) => {
        if let Err(e) = $result {
            let wrapped = error::ReadLoopError::from(e);
            // Write the error to hio
            let mut stdout = hio::hstdout().unwrap();
            writeln!(stdout, "Error: {:#?}", wrapped).unwrap();
            //
            // Store the error if it is the first one that occured
            if $storage.is_none() {
                $storage = Some(wrapped);
            }
        }
    }
}

entry!(main);

fn main() -> ! {
    let mut last_error: Option<error::ReadLoopError> = None;

    // These errors are unrecoverable so we do not save any errors here
    let p = stm32f103xx::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
    let mut afio = p.AFIO.constrain(&mut rcc.apb2);

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // set up esp8266
    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10.into_floating_input(&mut gpioa.crh);
    let esp_reset = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);
    let esp_timer = Timer::tim2(p.TIM2, Hertz(1), clocks, &mut rcc.apb1);

    let serial = Serial::usart1(
        p.USART1,
        (tx, rx),
        &mut afio.mapr,
        9600.bps(),
        clocks,
        &mut rcc.apb2,
    );
    let (tx, rx) = serial.split();
    let mut esp8266 = esp8266::Esp8266::new(tx, rx, esp_timer, esp_reset)
        .expect("Failed to initialise esp8266");


    let mut dhtxx_debug_pin = gpioa.pa2.into_push_pull_output(&mut gpioa.crl);


    let ane_timer = Timer::tim3(p.TIM3, Hertz(1), clocks, &mut rcc.apb1);
    // TODO: Use internal pull up instead
    let ane_pin = gpioa.pa1.into_floating_input(&mut gpioa.crl);
    let mut anemometer = anemometer::Anemometer::new(ane_pin, ane_timer, Second(15), 3);

    let mut dhtxx_pin = gpioa.pa0.into_push_pull_output(&mut gpioa.crl);
    let mut dhtxx = dhtxx::Dhtxx::new();

    let mut misc_timer = Timer::tim4(p.TIM4, Hertz(1), clocks, &mut rcc.apb1);

    // esp8266.communicate("+CWJAP?").unwrap();

    loop {
        // Try to send an error message if an error occured
        let should_clear_error = if let Some(ref err) = last_error {
            let mut buff = arrayvec::ArrayString::<[u8; 256]>::new();
            writeln!(buff, ";Fatal: {:?}", err)
                .expect("Fatal: Failed to send previous error to server");

            let send_result = esp8266.send_data(
                esp8266::ConnectionType::Tcp,
                IP_ADDRESS,
                2000,
                &buff
            );

            match send_result {
                Ok(()) => true,
                Err(_) => false
            }
        }
        else {
            false
        };

        if should_clear_error {
            last_error = None;
        }

        let (returned_pin, result) = read_and_send_dht_data(
            &mut esp8266,
            &mut dhtxx,
            dhtxx_pin,
            &mut gpioa.crl,
            &mut misc_timer,
            &mut dhtxx_debug_pin
        );
        dhtxx_pin = returned_pin;
        handle_result!(result, last_error);
        handle_result!(read_and_send_wind_speed(&mut esp8266, &mut anemometer), last_error);

        for _i in 0..SLEEP_ITERATIONS {
            esp8266.power_down();
            misc_timer.start_real(WAKEUP_INTERVAL);
            // misc_timer.listen(stm32f103xx_hal::timer::Event::Update);
            // asm::wfi();
            block!(misc_timer.wait()).unwrap();
            esp8266.power_up().expect("Failed to power up esp8266");
        }
    }
}

fn read_and_send_wind_speed(
    esp8266: &mut types::EspType,
    anemometer: &mut types::AnemometerType
) -> Result<(), error::AnemometerError>{
    let result = anemometer.measure();

    let mut encoding_buffer = arrayvec::ArrayString::<[_;32]>::new();
    communication::encode_f32("wind_raw", result, &mut encoding_buffer)?;

    Ok(esp8266.send_data(
        esp8266::ConnectionType::Tcp,
        IP_ADDRESS,
        2000,
        &encoding_buffer
    )?)
}

fn read_and_send_dht_data(
    esp8266: &mut types::EspType,
    dht: &mut types::DhtType,
    pin: dhtxx::OutPin,
    crl: &mut CRL,
    timer: &mut Timer<TIM4>,
    debug_pin: &mut dhtxx::DebugPin
) -> (dhtxx::OutPin, Result<(), error::DhtError>) {
    let (pin, reading) = dht.make_reading(pin, crl, timer, debug_pin);

    match reading {
        Ok(reading) => (pin, send_dht_data(esp8266, reading)),
        Err(e) => (pin, Err(error::DhtError::Dht(e)))
    }
}

fn send_dht_data(esp8266: &mut types::EspType, reading: dhtxx::Reading) -> Result<(), error::DhtError> {
    {
        let mut encoding_buffer = arrayvec::ArrayString::<[_;32]>::new();
        communication::encode_f32("humidity", reading.humidity, &mut encoding_buffer)?;

        // let a = 0;
        esp8266.send_data(
            esp8266::ConnectionType::Tcp,
            IP_ADDRESS,
            2000,
            &encoding_buffer
        )?;
    }
    {
        let mut encoding_buffer = arrayvec::ArrayString::<[_;32]>::new();
        communication::encode_f32("temperature", reading.temperature, &mut encoding_buffer)?;

        // let a = 0;
        esp8266.send_data(
            esp8266::ConnectionType::Tcp,
            IP_ADDRESS,
            2000,
            &encoding_buffer
        )?;
    }

    Ok(())
}


// define the hard fault handler
exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

// define the default exception handler
exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
/*
   Pinout:

   USART1_TX: PA9
   USART1_RX: PA10
 */
