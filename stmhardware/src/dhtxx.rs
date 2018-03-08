use stm32f103xx_hal::gpio::{Output, Input, PushPull, Floating};
use stm32f103xx_hal::gpio::gpioa::{PA1, CRL};
use stm32f103xx_hal::time::{MonoTimer, Hertz};
use stm32f103xx_hal::timer::Timer;
use stm32f103xx::TIM4;
use hal::prelude::*;

use embedded_hal_time::{RealCountDown, Microsecond, Millisecond};

const TIMEOUT_PADDING: u32 = 5;

pub type OutPin = PA1<Output<PushPull>>;
type InPin = PA1<Input<Floating>>;

#[derive(Debug)]
pub enum Error {
    Timeout,
    UnexpectedBitDuration
}

pub struct Dhtxx<T> 
where
    T: RealCountDown<Microsecond> + RealCountDown<Millisecond>
{
    mono_timer: MonoTimer,
    countdown_timer: T
}

pub struct Reading {
    temperature: u16,
    humidity: u16
}

impl<T> Dhtxx<T>
where
    T: RealCountDown<Microsecond> + RealCountDown<Millisecond>
{
    pub fn new(mono_timer: MonoTimer, countdown_timer: T) -> Self {
        Self {mono_timer, countdown_timer}
    }

    pub fn make_reading(&mut self, mut pin: OutPin, pin_ctrl: &mut CRL)
        -> Result<(Reading, OutPin), Error>
    {
        // Set low for 18 ms
        pin.set_low();
        self.wait_for_ms(Millisecond(18));
        // Pull up voltage, wait for 20us
        let mut pin = pin.into_floating_input(pin_ctrl);
        // Wait for input to go low. 20us timeout
        self.wait_for_pin_with_timeout(&pin, false, Microsecond(40 + TIMEOUT_PADDING))?;
        // // Wait for high in 80 us
        self.wait_for_pin_with_timeout(&pin, true, Microsecond(80 + TIMEOUT_PADDING))?;
        // // Wait for low in 80 us
        self.wait_for_pin_with_timeout(&pin, false, Microsecond(80 + TIMEOUT_PADDING))?;
        // Start of actual data
        let data = self.read_data(&pin)?;
        // XXX Sleep for a while to allow debugging
        self.wait_for_ms(Millisecond(2000));
        let pin = pin.into_push_pull_output(pin_ctrl);
        Ok((Reading{temperature: 0, humidity: 0}, pin.into_push_pull_output(pin_ctrl)))
    }

    fn read_data(&mut self, pin: &InPin) -> Result<[u8; 4], Error> {
        let mut data = [0;4];
        let frequency = self.mono_timer.frequency();
        let tick_length_us: f32 = 1_000_000.0 / frequency.0 as f32;

        for byte in 0..4 {
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
                    data[byte] |= !(1 << index);
                }
                else if let Ok(_) = self.wait_for_pin_with_timeout(&pin, false, Microsecond(70-28)) {
                    data[byte] &= (1 << index);
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

