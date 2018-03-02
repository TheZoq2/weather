use hal::{timer, digital};
use nb;

pub struct Anemometer<Pin, Timer>
where
    Pin: digital::InputPin,
    Timer: timer::CountDown,
    Timer::Time: Copy
{
    timer: Timer,
    pin: Pin,
    duration: (Timer::Time, u8)
}

impl<Pin, Timer> Anemometer<Pin, Timer>
where
    Pin: digital::InputPin,
    Timer: timer::CountDown,
    Timer::Time: Copy
{
    pub fn new(pin: Pin, timer: Timer, duration: (Timer::Time, u8)) -> Self {
        Self {
            pin,
            timer,
            duration
        }
    }

    pub fn measure(&mut self) -> f32 {
        let (timeout, initial_multiplyer) = self.duration;

        let mut multiplyer = initial_multiplyer;
        let mut last_state = self.pin.is_high();
        let mut cycle_count = 0;
        while multiplyer > 0 {
            self.timer.start(timeout);
            multiplyer -= 1;

            while let Err(nb::Error::WouldBlock) = self.timer.wait() {
                if last_state == false && self.pin.is_high() {
                    cycle_count += 1;
                }
                last_state = self.pin.is_high();
            }
        }

        return cycle_count as f32 / initial_multiplyer as f32;
    }
}
