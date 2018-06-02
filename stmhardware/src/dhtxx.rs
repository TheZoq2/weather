use stm32f103xx_hal::gpio::{Output, Input, PushPull, Floating};
use stm32f103xx_hal::gpio::gpioa::{PA1, CRL};
use stm32f103xx_hal::gpio::gpiob::{PB12};
use stm32f103xx_hal::time::{MonoTimer, Hertz};
use stm32f103xx_hal::timer::Timer;
use stm32f103xx::TIM4;
use hal::prelude::*;

use embedded_hal_time::{RealCountDown, Microsecond, Millisecond};

const TIMEOUT_PADDING: u32 = 5;

pub type OutPin = PA1<Output<PushPull>>;
pub type DebugPin = PB12<Output<PushPull>>;
type InPin = PA1<Input<Floating>>;

#[derive(Debug)]
pub enum Error {
    Timeout,
    UnexpectedBitDuration,
    IncorrectParity
}

pub struct Dhtxx<T> 
where
    T: RealCountDown<Microsecond> + RealCountDown<Millisecond>
{
    mono_timer: MonoTimer,
    countdown_timer: T
}

pub struct Reading {
    pub temperature: f32,
    pub humidity: f32
}

impl<T> Dhtxx<T>
where
    T: RealCountDown<Microsecond> + RealCountDown<Millisecond>
{
    pub fn new(mono_timer: MonoTimer, countdown_timer: T) -> Self {
        Self {mono_timer, countdown_timer}
    }

    pub fn make_reading(&mut self, mut pin: OutPin, pin_ctrl: &mut CRL, debug_pin: &mut DebugPin)
        -> Result<(Reading, OutPin), Error>
    {
        // Set low for 18 ms
        pin.set_low();
        self.wait_for_ms(Millisecond(18));
        // Pull up voltage, wait for 20us
        let pin = pin.into_floating_input(pin_ctrl);
        // Wait for input to go low. 20us timeout
        self.wait_for_pin_with_timeout(&pin, false, Microsecond(40 + TIMEOUT_PADDING))?;
        debug_pin.set_low();
        // // Wait for high in 80 us
        self.wait_for_pin_with_timeout(&pin, true, Microsecond(80 + TIMEOUT_PADDING))?;
        debug_pin.set_high();
        // // Wait for low in 80 us
        self.wait_for_pin_with_timeout(&pin, false, Microsecond(80 + TIMEOUT_PADDING))?;
        debug_pin.set_low();
        // Start of actual data
        let data = self.read_data(&pin)?;

        let reading = decode_dht_data(&data)?;
        // XXX Sleep for a while to allow debugging
        let pin = pin.into_push_pull_output(pin_ctrl);
        Ok((reading, pin.into_push_pull_output(pin_ctrl)))
    }

    fn read_data(&mut self, pin: &InPin) -> Result<[u8; 5], Error> {
        let mut data = [0;5];

        for byte in 0..5 {
            for index in 0..8 {
                // Wait for the pin to go high
                match self.wait_for_pin_with_timeout(&pin, true, Microsecond(50 + TIMEOUT_PADDING)) {
                    Ok(_) => {},
                    Err(e) => {
                        return Err(e);
                    }
                }

                // Wait for the pin to go low. If it does in 28 us this bit is a 0
                if let Ok(_) = self.wait_for_pin_with_timeout(&pin, false, Microsecond(28)) {
                    // data[byte] &= ~(1 << index);
                    data[byte] = (data[byte] << 1);
                }
                else if let Ok(_) = self.wait_for_pin_with_timeout(&pin, false, Microsecond(70-28)) {
                    data[byte] = (data[byte] << 1) | 1;
                }
                else {
                    return Err(Error::Timeout);
                }
            }
        }
        Ok(data)
    }

    fn wait_for_pin_with_timeout(
        &mut self,
        pin: &InPin,
        pin_high: bool,
        timeout: Microsecond,
    ) -> Result<(), Error> {
        self.countdown_timer.start_real(timeout);

        loop {
            if pin.is_high() == pin_high {
                return Ok(());
            }
            if self.countdown_timer.wait() == Ok(()) {
                return Err(Error::Timeout)
            }
        }
    }

    fn wait_for_us(&mut self, timeout: Microsecond) {
        self.countdown_timer.start_real(timeout);

        // Result<, !> can be safely unwrapped
        block!(self.countdown_timer.wait()).unwrap();
    }
    fn wait_for_ms(&mut self, timeout: Millisecond) {
        self.countdown_timer.start_real(timeout);

        // Result<, !> can be safely unwrapped
        block!(self.countdown_timer.wait()).unwrap();
    }
}


/**
  Decodes a byte sequence received from a dhtxx sensor.

  https://github.com/adafruit/DHT-sensor-library/blob/master/DHT.cpp
*/
fn decode_dht_data(data: &[u8;5]) -> Result<Reading, Error> {
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
    let humidity = data[0] as f32 + ((data[1] as f32) * 0.1);

    // Absolute value of the temperature is the 15 least significant bits of the reading
    // let temperature_abs = (((data[2] & 0x7f) as u16) << 8) + (data[3] as u16);
    let temperature_abs = (data[2] & 0x7f) as f32 + ((data[3] as f32) * 0.1);

    // The sign of the temperature is negative if the msb is 1
    let temperature_negative = (data[2] & 0b10000000) == 0b10000000;

    // Apply the sign
    let temperature = if temperature_negative {
        -temperature_abs
    }
    else {
        temperature_abs
    };

    Ok(Reading{humidity, temperature})
}
