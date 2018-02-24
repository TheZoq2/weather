use hal::{timer, digital};
use nb;

pub struct Anemometer<Pin, Timer, Duration>
where
    Pin: digital::InputPin,
    Timer: timer::CountDown,
    Duration: Fn() -> (Timer::Time, u8)
{
    timer: Timer,
    pin: Pin,
    duration: Duration
}

impl<Pin, Timer, Duration> Anemometer<Pin, Timer, Duration>
where
    Pin: digital::InputPin,
    Timer: timer::CountDown,
    Duration: Fn() -> (Timer::Time, u8)
{
    pub fn new(pin: Pin, timer: Timer, duration: Duration) -> Self {
        Self {
            pin,
            timer,
            duration
        }
    }

    pub fn measure(&mut self) -> f32 {
        let (_, initial_multiplyer) = (self.duration)();

        let mut multiplyer = initial_multiplyer;
        let mut last_state = self.pin.is_high();
        let mut cycle_count = 0;
        while multiplyer > 0 {
            let (timeout, _) = (self.duration)();
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
