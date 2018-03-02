use stm32f103xx_hal::gpio::{Output, Input, PushPull, Floating};
use stm32f103xx_hal::gpio::gpioa::{PA1, CRL};
use stm32f103xx_hal::time::{MonoTimer, Hertz};
use stm32f103xx_hal::timer::Timer;
use stm32f103xx::TIM4;
use hal::prelude::*;

use embedded_hal_time::{RealCountDown, Microsecond, Millisecond};

pub type OutPin = PA1<Output<PushPull>>;
type InPin = PA1<Input<Floating>>;

#[derive(Debug)]
pub enum Error {
    Timeout,
    _test
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
        const TIMEOUT_PADDING: u32 = 5;
        // Set low for 18 ms
        pin.set_low();
        self.wait_for_us(Microsecond(10000));
        // Pull up voltage, wait for 20us
        pin.set_high();
        self.wait_for_us(Microsecond(5000));
        // Reconfigure as input
        // let pin = pin.into_floating_input(pin_ctrl);
        // Wait for input to go low. 20us timeout
        // self.wait_for_pin_with_timeout(&pin, false, Microsecond(20 + TIMEOUT_PADDING))?;
        // // Wait for high in 80 us
        // self.wait_for_pin_with_timeout(&pin, true, Microsecond(80 + TIMEOUT_PADDING))?;
        // // Wait for low in 80 us
        // self.wait_for_pin_with_timeout(&pin, false, Microsecond(80 + TIMEOUT_PADDING))?;
        // // Start of actual data
        //
        Ok((Reading{temperature: 0, humidity: 0}, pin.into_push_pull_output(pin_ctrl)))
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
        // Convert timeout to hertz so we can use the timer
        self.countdown_timer.start_real(timeout);

        // Result<, !> can be safely unwrapped
        block!(self.countdown_timer.wait()).unwrap();
    }
    fn wait_for_ms(&mut self, timeout: u32) {
        // Convert timeout to hertz so we can use the timer
        let timeout_us = timeout * 1000;
        self.countdown_timer.start_real(Microsecond(timeout_us));

        // Result<, !> can be safely unwrapped
        block!(self.countdown_timer.wait()).unwrap();
    }

    fn read_high_time() {
        
    }
}

