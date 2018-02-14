#![feature(proc_macro)]
#![no_std]
#![allow(dead_code)]

extern crate f3;
#[macro_use(block)]
extern crate nb;

extern crate cortex_m_rtfm as rtfm;

// #[macro_use]
// extern crate cortex_m;
extern crate cortex_m_semihosting;

extern crate embedded_hal as hal;
extern crate stm32f30x_hal;

use stm32f30x_hal::prelude::*;
use stm32f30x_hal::serial::{Event, Serial};
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

    let mut timer = Timer::tim2(p.TIM2, Hertz(1), clocks, &mut rcc.apb1);

    let serial = Serial::usart1(
        p.USART1,
        (tx, rx),
        9600.bps(),
        clocks,
        &mut rcc.apb2,
    );
    let (mut tx, mut rx) = serial.split();

    // Disable echo to avoid having the serial be overrun
    esp8266::send_at_command(&mut tx, "E0");
    timer.start(Hertz(1));
    block!(timer.wait());
    let _response = esp8266::wait_for_at_reply(&mut rx, &mut timer, || Hertz(1));

    esp8266::send_at_command(&mut tx, "+GMR").unwrap();

    let response = esp8266::wait_for_at_reply(&mut rx, &mut timer, || Hertz(1));

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
