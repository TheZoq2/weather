use stm32f1xx_hal::gpio::{Output, Input, PushPull, Floating};
use stm32f1xx_hal::gpio::gpioa::{PA0, PA2, CRL};
use embedded_hal::prelude::*;

use embedded_hal_time::{RealCountDown, Microsecond, Millisecond};

const TIMEOUT_PADDING: u32 = 15;

pub type OutPin = PA0<Output<PushPull>>;
type InPin = PA0<Input<Floating>>;

pub type DebugPin = PA2<Output<PushPull>>;


#[derive(Debug)]
pub enum Error {
    Timeout,
    UnexpectedBitDuration,
    IncorrectParity
}

pub struct Dhtxx {}

pub struct Reading {
    pub temperature: f32,
    pub humidity: f32
}


pub trait DhtTimer: RealCountDown<Microsecond> + RealCountDown<Millisecond> { }
impl<T> DhtTimer for T where T: RealCountDown<Microsecond> + RealCountDown<Millisecond> {}

impl Dhtxx
{
    pub fn new() -> Self {
        Self {}
    }

    pub fn make_reading(&mut self, mut pin: OutPin, pin_ctrl: &mut CRL, timer: &mut impl DhtTimer, debug_pin: &mut DebugPin)
        -> (OutPin, Result<Reading, Error>)
    {
        self.send_init(&mut pin, timer);
        let pin = pin.into_floating_input(pin_ctrl);

        let read_result = self.full_read_sequence(&pin, timer, debug_pin);

        let mut pin = pin.into_push_pull_output(pin_ctrl);
        pin.set_high();
        (pin, read_result)
    }

    fn send_init(&self, pin: &mut OutPin, timer: &mut impl DhtTimer) {
        // Set low for 18 ms
        pin.set_low();
        wait_for_ms(Millisecond(18), timer);
    }

    fn full_read_sequence(&mut self, pin: &InPin, timer: &mut impl DhtTimer, debug_pin: &mut DebugPin) -> Result<Reading, Error> {
        // Wait for input to go low. 20us timeout
        wait_for_pin_with_timeout(&pin, false, Microsecond(40 + TIMEOUT_PADDING), timer)?;
        // // Wait for high in 80 us
        wait_for_pin_with_timeout(&pin, true, Microsecond(80 + TIMEOUT_PADDING), timer)?;
        // // Wait for low in 80 us
        wait_for_pin_with_timeout(&pin, false, Microsecond(80 + TIMEOUT_PADDING), timer)?;
        // Start of actual data
        let data = self.read_data(&pin, timer, debug_pin)?;

        decode_dht_data(data)
    }


    fn read_data(&mut self, pin: &InPin, timer: &mut impl DhtTimer, debug_pin: &mut DebugPin) -> Result<[u8; 5], Error> 
    {
        let mut data = [0;5];


        for byte in 0..5 {
            for _ in 0..8 {
                debug_pin.set_low();
                // Wait for the pin to go high
                match wait_for_pin_with_timeout(&pin, true, Microsecond(50 + TIMEOUT_PADDING), timer) {
                    Ok(_) => {},
                    Err(e) => {
                        return Err(e);
                    }
                }
                debug_pin.set_high();

                // Wait for the pin to go low. If it does in 28 us this bit is a 0
                if wait_for_pin_with_timeout(&pin, false, Microsecond(28 + TIMEOUT_PADDING), timer).is_ok() {
                    // data[byte] &= ~(1 << index);
                    data[byte] <<= 1;
                    debug_pin.set_low();
                }
                else if wait_for_pin_with_timeout(&pin, false, Microsecond(70-28 + TIMEOUT_PADDING * 2), timer).is_ok() {
                    data[byte] = (data[byte] << 1) | 1;
                    debug_pin.set_low();
                }
                else {
                    return Err(Error::Timeout);
                }
            }
        }
        Ok(data)
    }
}

fn wait_for_pin_with_timeout<T>(pin: &InPin, pin_high: bool, timeout: Microsecond, timer: &mut T)
    -> Result<(), Error>
    where
        T: DhtTimer,
{
    timer.start_real(timeout);

    loop {
        if pin.is_high() == pin_high {
            return Ok(());
        }
        if timer.wait() == Ok(()) {
            return Err(Error::Timeout)
        }
    }
}

fn wait_for_ms<T>(timeout: Millisecond, timer: &mut T)
where
    T: DhtTimer
{
    timer.start_real(timeout);

    // Result<, !> can be safely unwrapped
    block!(timer.wait()).unwrap();
}


/**
  Decodes a byte sequence received from a dhtxx sensor.

  https://github.com/adafruit/DHT-sensor-library/blob/master/DHT.cpp
*/
fn decode_dht_data(data: [u8;5]) -> Result<Reading, Error> {
    // Check the parity bit
    let mut parity: u8 = 0;
    for bit in 0..data.len()-1 {
        parity += data[bit];
    }

    if parity != data[data.len()-1] {
        return Err(Error::IncorrectParity)
    }

    // Humidity is simply a 16 bit number
    // let humidity = ((data[0] as u16) << 8) + (data[1] as u16);
    let humidity = (256. * data[0] as f32 + (data[1] as f32)) * 0.1;

    // Absolute value of the temperature is the 15 least significant bits of the reading
    // let temperature_abs = (((data[2] & 0x7f) as u16) << 8) + (data[3] as u16);
    let temperature_abs = (((data[2] & 0x7f) as f32) * 256. + (data[3] as f32)) * 0.1;

    // The sign of the temperature is negative if the msb is 1
    let temperature_negative = (data[2] & 0b1000_0000) == 0b1000_0000;

    // Apply the sign
    let temperature = if temperature_negative {
        -temperature_abs
    }
    else {
        temperature_abs
    };

    Ok(Reading{humidity, temperature})
}
