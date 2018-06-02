use hal::{timer, digital};
use embedded_hal_time::{RealCountDown, Second};

use nb;


pub struct Anemometer<Pin, Timer>
where
    Pin: digital::InputPin,
    Timer: RealCountDown<Second>,
{
    timer: Timer,
    pin: Pin,
    measurement_duration: Second,
    switches_per_cycle: u8
}

impl<Pin, Timer> Anemometer<Pin, Timer>
where
    Pin: digital::InputPin,
    Timer: RealCountDown<Second>,
{
    pub fn new(pin: Pin, timer: Timer, measurement_duration: Second, switches_per_cycle: u8) -> Self {
        Self {
            pin,
            timer,
            measurement_duration,
            switches_per_cycle
        }
    }

    pub fn measure(&mut self) -> f32 {
        let mut last_state = self.pin.is_high();
        let mut cycle_count = 0;
        self.timer.start_real(self.measurement_duration);

        while let Err(nb::Error::WouldBlock) = self.timer.wait() {
            if last_state == false && self.pin.is_high() {
                cycle_count += 1;
            }
            last_state = self.pin.is_high();
        }

        return cycle_count as f32 / (self.switches_per_cycle as f32 + self.measurement_duration.0 as f32);
    }
}
