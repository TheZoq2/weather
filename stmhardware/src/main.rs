#![feature(proc_macro)]
#![no_std]
#![allow(dead_code)]
#![feature(generic_associated_types)]

extern crate f3;
#[macro_use(block)]
extern crate nb;

extern crate cortex_m_rtfm as rtfm;

// #[macro_use]
// extern crate cortex_m;
extern crate cortex_m_semihosting;

extern crate embedded_hal as hal;
extern crate stm32f30x_hal;
extern crate arrayvec;

use stm32f30x_hal::prelude::*;
use stm32f30x_hal::serial::{Serial};
use stm32f30x_hal::stm32f30x::{self};
use stm32f30x_hal::time::Hertz;
use stm32f30x_hal::timer::Timer;

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

    let timer = Timer::tim2(p.TIM2, Hertz(1), clocks, &mut rcc.apb1);

    let serial = Serial::usart1(
        p.USART1,
        (tx, rx),
        9600.bps(),
        clocks,
        &mut rcc.apb2,
    );
    let (tx, rx) = serial.split();

    let mut esp8266 = esp8266::Esp8266::new(tx, rx, timer, || (Hertz(1), 3)).unwrap();

    esp8266.communicate("+CWJAP?").unwrap();

    loop {
        match esp8266.send_data(esp8266::ConnectionType::Tcp, "192.168.1.5", 2000, "temperature:500") {
            Ok(val) => {},
            Err(e) => {
                esp8266.close_connection().unwrap();
                panic!();
            }
        }
    }


    loop {
        rtfm::wfi();
    }
}


/*
fn on_timer(_: &mut Threshold, mut r: TIM6_DACUNDER::Resources) {
    // Clear flag to avoid getting stuck in interrupt
    r.TIMER.wait().unwrap();

    let result = r.AT_BUFFER.parse();
    match result {
        Some(status) => {
            writeln!(hio::hstdout().unwrap(), "Got status: {:?}", status).unwrap();
        },
        None => {
            writeln!(hio::hstdout().unwrap(), "Not an AT command").unwrap();
        }
    }
}
*/

/*
   Pinout:

   USART1_TX: PA9
   USART1_RX: PA10
 */
