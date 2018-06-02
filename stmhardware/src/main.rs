#![feature(proc_macro)]
#![no_main]
#![no_std]
#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(generic_associated_types)]
#![feature(start)]

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

use cortex_m::asm;
use rt::ExceptionFrame;

use stm32f103xx_hal::prelude::*;
use stm32f103xx_hal::serial::{Serial};
//use stm32f103xx_hal::stm32f103xx::{self};
use stm32f103xx_hal::time::{Hertz, MonoTimer};
use stm32f103xx_hal::timer::Timer;
use stm32f103xx_hal::gpio::gpioa::{CRL};
use stm32f103xx_hal::gpio::gpiob;
use embedded_hal_time::{RealCountDown, Microsecond};

mod serial;
mod esp8266;
mod anemometer;
mod communication;
mod api;
mod dhtxx;
mod types;

entry!(main);

fn main() -> ! {
    let p = stm32f103xx::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
    let mut afio = p.AFIO.constrain(&mut rcc.apb2);

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10.into_floating_input(&mut gpioa.crh);


    let timer = Timer::tim2(p.TIM2, Hertz(1), clocks, &mut rcc.apb1);

    let serial = Serial::usart1(
        p.USART1,
        (tx, rx),
        &mut afio.mapr,
        9600.bps(),
        clocks,
        &mut rcc.apb2,
    );
    let (tx, rx) = serial.split();

    let mut esp8266 = esp8266::Esp8266::new(tx, rx, timer, (Hertz(1), 3)).unwrap();

    let ane_timer = Timer::tim3(p.TIM3, Hertz(1), clocks, &mut rcc.apb1);
    // TODO: Use internal pull up instead
    let ane_pin = gpioa.pa0.into_floating_input(&mut gpioa.crl);
    let ane_timeout = (Hertz(1), 5);
    let mut anemometer = anemometer::Anemometer::new(ane_pin, ane_timer, ane_timeout);

    let mut dhtxx_pin = gpioa.pa1.into_push_pull_output(&mut gpioa.crl);
    let dhtxx_timer = Timer::tim4(p.TIM4, Hertz(1), clocks, &mut rcc.apb1);
    let mut dhtxx = dhtxx::Dhtxx::new(dhtxx_timer);

    let mut debug_pin = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);
    debug_pin.set_high();


    // esp8266.communicate("+CWJAP?").unwrap();

    loop {
        dhtxx_pin = read_and_send_dht_data(&mut esp8266, &mut dhtxx, dhtxx_pin, &mut gpioa.crl, &mut debug_pin);
        read_and_send_wind_speed(&mut esp8266, &mut anemometer);
        loop {}
    }
}

fn read_and_send_wind_speed(esp8266: &mut types::EspType, anemometer: &mut types::AnemometerType){
    let result = anemometer.measure();

    let mut encoding_buffer = arrayvec::ArrayString::<[_;32]>::new();
    communication::encode_i32("wind_raw", (result * 10.) as i32, &mut encoding_buffer)
        .unwrap();

    // let a = 0;
    let send_result = esp8266.send_data(
        esp8266::ConnectionType::Tcp,
        "192.168.1.5",
        2000,
        &encoding_buffer
    );

    match send_result {
        Ok(_) => {},
        Err(_e) => {
            //Something went wrong. Trying to close connection as cleanup
            esp8266.close_connection().ok();
        }
    }
}

fn read_and_send_dht_data(
    esp8266: &mut types::EspType,
    dht: &mut types::DhtType,
    pin: dhtxx::OutPin,
    crl: &mut CRL,
    debug_pin: &mut dhtxx::DebugPin
) -> dhtxx::OutPin {
    let (reading, pin) = dht.make_reading(pin, crl, debug_pin).unwrap();

    {
        let mut encoding_buffer = arrayvec::ArrayString::<[_;32]>::new();
        communication::encode_f32("temperature", reading.temperature, &mut encoding_buffer)
            .expect("Failed to encode temperature");

        // let a = 0;
        esp8266.send_data(
            esp8266::ConnectionType::Tcp,
            "192.168.1.5",
            2000,
            &encoding_buffer
        ).expect("Failed to send temperature reading");
    }

    {
        let mut encoding_buffer = arrayvec::ArrayString::<[_;32]>::new();
        communication::encode_f32("humidity", reading.humidity, &mut encoding_buffer)
            .unwrap();

        // let a = 0;
        esp8266.send_data(
            esp8266::ConnectionType::Tcp,
            "192.168.1.5",
            2000,
            &encoding_buffer
        ).expect("Failed to send humidity reading");
    }

    pin
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
