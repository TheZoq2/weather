use stm32f103xx_hal::gpio::{Output, Input, PushPull, Floating};
use stm32f103xx_hal::gpio::gpioa::{PA1, CRL};
use stm32f103xx_hal::time::{MonoTimer, Hertz};
use stm32f103xx_hal::timer::Timer;
use stm32f103xx::TIM4;
use hal::prelude::*;

type InPin = PA1<Input<Floating>>;
type OutPin = PA1<Output<PushPull>>;

enum Error {
    Timeout
}

pub struct Dhtxx {
    mono_timer: MonoTimer,
    countdown_timer: Timer<TIM4>
}

struct Reading {

}

impl Dhtxx {
    pub fn new(mono_timer: MonoTimer, countdown_timer: Timer<TIM4>) -> Self {
        Self {mono_timer, countdown_timer}
    }
    fn make_reading(&mut self, mut pin: OutPin, pin_ctrl: &mut CRL)
        -> Result<(Reading, OutPin), Error>
    {
        const TIMEOUT_PADDING: u32 = 5;
        // Set low for 18 ms
        pin.set_low();
        self.wait_for_ms(18);
        // Pull up voltage, wait for 20us
        pin.set_high();
        self.wait_for_us(20);
        // Reconfigure as input
        let pin = pin.into_floating_input(pin_ctrl);
        // Wait for input to go low. 20us timeout
        self.wait_for_pin_with_timeout(&pin, false, 20 + TIMEOUT_PADDING)?;
        // Wait for high in 80 us
        self.wait_for_pin_with_timeout(&pin, true, 80 + TIMEOUT_PADDING)?;
        // Wait for low in 80 us
        self.wait_for_pin_with_timeout(&pin, false, 80 + TIMEOUT_PADDING)?;
        // Start of actual data
        //
        Ok((Reading{}, pin.into_push_pull_output(pin_ctrl)))
    }

    fn wait_for_pin_with_timeout(
        &mut self,
        pin: &InPin,
        pin_high: bool,
        timeout_us: u32,
    ) -> Result<(), Error> {
        // Convert timeout to hertz so we can use the timer
        let timeout_hz = 1_000_000 / timeout_us;

        self.countdown_timer.start(Hertz(timeout_hz));

        loop {
            if pin.is_high() == pin_high {
                return Ok(());
            }
            if self.countdown_timer.wait() == Ok(()) {
                return Err(Error::Timeout)
            }
        }
    }

    fn wait_for_us(&mut self, timeout_us: u32) {
        // Convert timeout to hertz so we can use the timer
        let timeout_hz = 1_000_000 / timeout_us;

        self.countdown_timer.start(Hertz(timeout_hz));

        // Result<, !> can be safely unwrapped
        block!(self.countdown_timer.wait()).unwrap();
    }
    fn wait_for_ms(&mut self, timeout_ms: u32) {
        // Convert timeout to hertz so we can use the timer
        let timeout_hz = 1_000/ timeout_ms;

        self.countdown_timer.start(Hertz(timeout_hz));

        // Result<, !> can be safely unwrapped
        block!(self.countdown_timer.wait()).unwrap();
    }

    fn read_high_time() {
        
    }
}

