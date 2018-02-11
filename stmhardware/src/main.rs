#![deny(unsafe_code)]
#![no_std]

extern crate f3;
#[macro_use(block)]
extern crate nb;

// #[macro_use]
// extern crate cortex_m;
extern crate cortex_m_semihosting;
use cortex_m_semihosting::hio;
use core::fmt::{Write, Display};

extern crate embedded_hal as hal;

use f3::hal::prelude::*;
use f3::hal::serial::Serial;
use f3::hal::time::Hertz;
use f3::hal::timer::Timer;
use f3::hal::stm32f30x;

mod serial;
mod esp8266;

fn main() {
    let p = stm32f30x::Peripherals::take().unwrap();
 
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let mut gpioa = p.GPIOA.split(&mut rcc.ahb);
 
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
 
    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
 
    // Init serial
    let serial = Serial::usart1(p.USART1, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb2);
    let (mut tx, mut rx) = serial.split();

    let mut timer = Timer::tim2(p.TIM2, Hertz(1), clocks, &mut rcc.apb1);
 
    loop {
        esp8266::send_at_command(&mut tx, "GMR").unwrap();
        let result = esp8266::read_at_response(&mut rx, &mut timer, || 1.hz());
        match result {
            Ok(esp8266::ATResponse::Ok) => {
                writeln!(hio::hstdout().unwrap(), "Got OK").unwrap();
            },
            Ok(_) => {
                writeln!(hio::hstdout().unwrap(), "Got other").unwrap();
            },
            Err(e) => {
                writeln!(hio::hstdout().unwrap(), "Got err: {:?}", e).unwrap();
            }
        }
    }
}


/*
   Pinout:

   USART1_TX: PA9
   USART1_RX: PA10
 */
