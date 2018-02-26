use stm32f103xx_hal::gpio::{Output, Input, PushPull, Floating};
use stm32f103xx_hal::gpio::gpioa::PA1;
use stm32f103xx_hal::time::MonoTimer;
use stm32f103xx_hal::timer::Timer;
use stm32f103xx::TIM3;

type InPin = PA1<Input<Floating>>;
type OutPin = PA1<Output<PushPull>>;

enum Error {
    Timeout
}

struct Dhtxx {
    pin: PA1<Output<PushPull>>,
    mono_timer: MonoTimer,
    countdown_timer: Timer<TIM3>
}

struct Reading {

}

impl Dhtxx {
    fn make_reading(&self, pin: OutPin) -> (Reading, OutPin) {
        // Configure the pin as an output
        // Set low for 18 ms
        // Pull up voltage, wait for 20us
        // Reconfigure as input
        // Wait for input to go low. 20us timeout
        // Wait for high in 80 us
        // Wait for low in 80 us
        // Start of actual data
        (Reading{}, pin)
    }

    fn wait_for_pin_with_timeout(
        &self,
        pin_high: bool,
        timeout_us: u32,
        pin: InPin
    ) -> Result<(), Error> {
        // Convert timeout to hertz so we can use the timer
        let timeout_hz = 1_000_000 / timeout_us;
    }
}

