#![feature(proc_macro)]
#![no_std]

extern crate f3;
#[macro_use(block)]
extern crate nb;

extern crate cortex_m_rtfm as rtfm;

// #[macro_use]
// extern crate cortex_m;
extern crate cortex_m_semihosting;
use cortex_m_semihosting::hio;
use core::fmt::{Write, Display};

extern crate embedded_hal as hal;

use f3::hal::prelude::*;
use f3::hal::serial::{Event, Serial, Rx, Tx};
use f3::hal::time::Hertz;
use f3::hal::timer::{self, Timer};
use f3::hal::stm32f30x::{self, USART1, TIM6};

use rtfm::{app, Threshold};

mod serial;
mod esp8266;
mod dma;

pub struct Buffer {
    pub data: [u8; 256],
    pub index: u8
}
impl Buffer {
    pub fn push(&mut self, byte: u8) {
        self.data[self.index as usize] = byte;
        self.index += 1;
    }
}

app! {
    device: stm32f30x,

    resources: {
        static TX: Tx<USART1>;
        static RX: Rx<USART1>;
        static TIMER: Timer<TIM6>;
        static USART_BUFFER: Buffer = Buffer{data: [0;256], index: 0};
    },

    idle: {
        resources: [TX]
    },

    tasks: {
        USART1_EXTI25: {
            path: on_usart,
            resources: [RX, USART_BUFFER, TIMER]
        },
        TIM6_DACUNDER: {
            path: on_timer,
            resources: [USART_BUFFER, TIMER]
        }
    },
}

fn init(p: init::Peripherals, r: init::Resources) -> init::LateResources {
    let mut flash = p.device.FLASH.constrain();
    let mut rcc = p.device.RCC.constrain();
    let mut gpioa = p.device.GPIOA.split(&mut rcc.ahb);

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);

    // Configure the serial port to the correct settings for the ESP8266
    // 1 stop bit
    p.device.USART1.cr2.write(|w| {
        unsafe {w.stop().bits(0)} // 1 stop bit
    });
    // 0 parity bits
    p.device.USART1.cr1.write(|w| {
        w.pce().clear_bit()
    });
    // 3 flow control TODO?

    let mut serial = Serial::usart1(
        p.device.USART1,
        (tx, rx),
        9600.bps(),
        clocks,
        &mut rcc.apb2,
    );
    serial.listen(Event::Rxne);
    let (tx, rx) = serial.split();

    // let atbuffer = esp8266::ATResponseBuffer::new();

    let mut timer = Timer::tim6(p.device.TIM6, 1.hz(), clocks, &mut rcc.apb1);
    timer.listen(timer::Event::TimeOut);

    init::LateResources {TX: tx, RX: rx, TIMER: timer}
}


fn idle(_: &mut Threshold, r: idle::Resources) -> ! {
    // Send the wifi init command
    esp8266::send_at_command(r.TX, "GMR").unwrap();

    // sleep
    loop {
        rtfm::wfi();
    }
}

fn on_usart(_: &mut Threshold, mut r: USART1_EXTI25::Resources) {
    // let read_result = r.RX.read().unwrap();

    // r.USART_BUFFER.push(read_result);
}

fn on_timer(_: &mut Threshold, mut r: TIM6_DACUNDER::Resources) {
    // Clear flag to avoid getting stuck in interrupt
    r.TIMER.wait().unwrap();

    let data = r.USART_BUFFER.data;
    let parsed = esp8266::parse_at_response(&data);
}

/*
   Pinout:

   USART1_TX: PA9
   USART1_RX: PA10
 */
