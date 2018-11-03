
#![no_main]
#![no_std]
#![allow(dead_code)]
#![allow(unused_imports)]
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

extern crate embedded_hal as hal; extern crate embedded_hal_time;
//extern crate stm32f30x_hal;
extern crate stm32f103xx_hal;
#[macro_use(interrupt)]
extern crate stm32f103xx;
extern crate arrayvec;
extern crate panic_semihosting;
extern crate itoa;
extern crate dhtxx;
extern crate void;

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
use stm32f103xx_hal::gpio::{Analog};
use stm32f103xx_hal::gpio::gpioa::{self, CRL};
use stm32f103xx_hal::gpio::gpiob;
use stm32f103xx_hal::adc::{AnalogPin, Adc};
use stm32f103xx_hal::rtc::Rtc;
use stm32f103xx::{TIM4, TIM3, ADC1};
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
// const IP_ADDRESS: &str = "192.168.0.12";
const PORT: u16 = 2000;
// const READ_INTERVAL: Second = Second(60*5);
// const READ_INTERVAL: Second = Second(30);
// Sleep for 5 minutes between reads, but wake up once every 10 seconds seconds
// to keep power banks from shutting down the psu
// const WAKEUP_INTERVAL: Second = Second(60*5);
const WAKEUP_INTERVAL: Second = Second(5);
// const SLEEP_ITERATIONS: u8 = 6;
const SLEEP_ITERATIONS: u8 = 1;
const ADC_RESOLUTION: u16 = 4096;


macro_rules! handle_result {
    ($result:expr, $storage:ident, $esp:ident) => {
        if let Err(e) = $result {
            let wrapped = e.into();

            /*
            let _hio_result = hio::hstdout().map(|mut hio| {
                writeln!(
                    hio,
                    "ReadLoopError: {:?}",
                    wrapped
                )
            });
            */

            match send_loop_error(&mut $esp, &wrapped) {
                Ok(()) => {},
                Err(send_err) => {
                    /*
                    let _hio_result = hio::hstdout().map(|mut hio| {
                        writeln!(
                            hio,
                            "Failed to send error, storing for future: {:?}",
                            send_err
                        )
                    });
                    */

                    // Store the error if it is the first one that occured
                    if $storage.is_none() {
                        $storage = Some(wrapped);
                    }
                }
            };
        }
    }
}

interrupt!(RTCALARM, timer_interrupt);
// interrupt!(TIM3, timer_interrupt);
// static mut WAKEUP_TIMER: Option<Timer<TIM3>> = None;
static mut RTC: Option<Rtc> = None;

entry!(main);
fn main() -> ! {
    let mut last_error: Option<error::ReadLoopError> = None;

    // These errors are unrecoverable so we do not save any errors here
    let p = stm32f103xx::Peripherals::take().unwrap();
    let cp = stm32f103xx::CorePeripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
    let mut afio = p.AFIO.constrain(&mut rcc.apb2);
    let mut nvic = cp.NVIC;
    let mut scb = cp.SCB;

    let mut pwr = p.PWR;
    let mut exti = p.EXTI;

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
    // Power the device down because we don't need it right now
    esp8266.power_down();


    let mut dhtxx_debug_pin = gpioa.pa2.into_push_pull_output(&mut gpioa.crl);


    // let ane_timer = Timer::tim3(p.TIM3, Hertz(1), clocks, &mut rcc.apb1);
    // // TODO: Use internal pull up instead
    // let ane_pin = gpioa.pa1.into_floating_input(&mut gpioa.crl);
    // let mut anemometer = anemometer::Anemometer::new(ane_pin, ane_timer, Second(15), 3);

    let mut dhtxx_pin = gpioa.pa0.into_push_pull_output(&mut gpioa.crl);
    let mut dhtxx = dhtxx::Dhtxx::new();

    let mut misc_timer = Timer::tim4(p.TIM4, Hertz(1), clocks, &mut rcc.apb1);


    let mut battery_sens_pin = gpioa.pa1.into_analog_input(&mut gpioa.crl);
    let mut adc = Adc::adc1(p.ADC1, &mut rcc.apb2);

    unsafe {
        WAKEUP_TIMER = Some(Timer::tim3(p.TIM3, Hertz(1), clocks, &mut rcc.apb1));
    }
    nvic.enable(stm32f103xx::Interrupt::TIM3);

    // esp8266.communicate("+CWJAP?").unwrap();

    loop {
        // Try to send an error message if an error occured
        let should_clear_error = if let Some(ref err) = last_error {
            match send_loop_error(&mut esp8266, err) {
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

        let battery_level = read_battery_voltage(&battery_sens_pin, &mut adc);

        let (returned_pin, dht_read_result) = read_dht_data(
            &mut dhtxx,
            dhtxx_pin,
            &mut gpioa.crl,
            &mut misc_timer,
            &mut dhtxx_debug_pin
        );
        dhtxx_pin = returned_pin;

        handle_result!(esp8266.power_up(), last_error, esp8266);

        let dht_result = dht_read_result.map(|reading| {
            let send_result = send_dht_data(&mut esp8266, reading);
            handle_result!(send_result, last_error, esp8266);
        });
        handle_result!(dht_result, last_error, esp8266);
        handle_result!(send_battery_voltage(&mut esp8266, battery_level), last_error, esp8266);

        // handle_result!(read_and_send_wind_speed(&mut esp8266, &mut anemometer), last_error, esp8266);

        esp8266.power_down();
        for _i in 0..SLEEP_ITERATIONS {
            // esp8266.pull_some_current();
            // unsafe {
            //     WAKEUP_TIMER.as_mut().unwrap().start_real(WAKEUP_INTERVAL);
            //     WAKEUP_TIMER.as_mut().unwrap().listen(stm32f103xx_hal::timer::Event::Update);
            // }
            stop_mode(
                &mut exti_emr,
                &mut system_control_block,
                &mut pwr_cr,
                &mut rtc,
                10,
            );
            // asm::wfi();
            // block!(misc_timer.wait()).unwrap();
        }
        misc_timer.unlisten(stm32f103xx_hal::timer::Event::Update);
    }
}


fn timer_interrupt() {
    let mut cp = unsafe {
        stm32f103xx::CorePeripherals::steal()
    };
    cp.NVIC.clear_pending(stm32f103xx::Interrupt::TIM4);

    // Reset the interrupt
    unsafe {
        WAKEUP_TIMER.as_mut().unwrap().wait();
    }
}

fn send_loop_error(esp: &mut types::EspType, err: &error::ReadLoopError)
    -> Result<(), error::EspTransmissionError>
{
    let mut buff = arrayvec::ArrayString::<[u8; 256]>::new();
    writeln!(buff, ";Fatal: {:?}", err)
        .expect("Fatal: err string buff was too small in send_loop_error");

    send_data(esp, &buff)
}

fn send_data(esp: &mut types::EspType, data: &str) -> Result<(), error::EspTransmissionError> {
    esp.send_data(
        esp8266::ConnectionType::Tcp,
        IP_ADDRESS,
        PORT,
        &data
    )
}


fn read_and_send_wind_speed(
    esp8266: &mut types::EspType,
    anemometer: &mut types::AnemometerType
) -> Result<(), error::AnemometerError>{
    let result = anemometer.measure();

    let mut encoding_buffer = arrayvec::ArrayString::<[_;32]>::new();
    communication::encode_f32("wind_raw", result, &mut encoding_buffer)?;

    Ok(send_data(esp8266, &encoding_buffer)?)
}

fn read_dht_data(
    dht: &mut types::DhtType,
    pin: dhtxx::OutPin,
    crl: &mut CRL,
    timer: &mut Timer<TIM4>,
    debug_pin: &mut dhtxx::DebugPin
) -> (dhtxx::OutPin, Result<dhtxx::Reading, error::DhtError>) {
    let (pin, reading) = dht.make_reading(pin, crl, timer, debug_pin);

    match reading {
        Ok(reading) => (pin, Ok(reading)),
        Err(e) => (pin, Err(error::DhtError::Dht(e)))
    }
}

fn send_dht_data(esp8266: &mut types::EspType, reading: dhtxx::Reading) -> Result<(), error::DhtError> {
    {
        let mut encoding_buffer = arrayvec::ArrayString::<[_;32]>::new();
        communication::encode_f32("humidity", reading.humidity, &mut encoding_buffer)?;

        // let a = 0;
        send_data(esp8266, &encoding_buffer)?;
    }
    {
        let mut encoding_buffer = arrayvec::ArrayString::<[_;32]>::new();
        communication::encode_f32("temperature", reading.temperature, &mut encoding_buffer)?;

        // let a = 0;
        send_data(esp8266, &encoding_buffer)?;
    }

    Ok(())
}



fn read_battery_voltage(battery_sensor: &gpioa::PA1<Analog>, adc: &mut Adc<ADC1>) -> f32 {
    ((battery_sensor.analog_read(adc) as f32) / (ADC_RESOLUTION as f32) * 4.2) + 0.10
}

fn send_battery_voltage(esp: &mut types::EspType, voltage: f32) -> Result<(), error::BatteryReadError> {
    let mut encoding_buffer = arrayvec::ArrayString::<[_;32]>::new();
    communication::encode_f32("battery", voltage, &mut encoding_buffer)?;

    Ok(send_data(esp, &encoding_buffer)?)
}


/**
  Puts the CPU into stop mode. Before this is done, wakeup has to
  be configured. See page 75 of datasheet for info

  Additionally, all pending interrupts need to be clear

  To exit stop mode
*/
fn stop_mode(
    exti_emr: &mut stm32f103xx::exti::EMR,
    system_control_block: &mut cortex_m::peripheral::SCB,
    pwr_control: &mut stm32f103xx::pwr::CR,
    rtc: &mut Rtc,
    time_seconds: u32
) {
    rtc.set_alarm(time_seconds);

    // Set SLEEPDEEP in cortex-m3 system control register
    unsafe {
        system_control_block.scr.modify(|prev_value| {
            // Write a 1 in SLEEPDEEP flag
            // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0552a/Cihhjgdh.html
            prev_value | 0b100
        });
    }

    // Clear PDDS bit in PWR_CR to enable stop mode
    // Set voltage regulator mode using LDPS in PWR_CR
    pwr_control.modify(|_r, w| {
        // Enable stop mode
        w.pdds().clear_bit()
        // Voltage regulators to low power mode
         .lpds().set_bit()
    });

    // Enable RTCAlarm event
    exti_emr.modify(|_r, w| w.mr17().set_bit());
    // Maybe set rising or falling edge as well

    // Call asm::wfi() or asm::wfe()
    asm::wfe();
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
