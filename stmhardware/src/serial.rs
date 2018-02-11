extern crate embedded_hal as hal;
extern crate nb;

use hal::prelude::*;

#[derive(Debug)]
pub enum Error<E> {
    /// Serial interface error
    Serial(E),
    TimedOut,
}

pub fn read_with_timeout<S, T>(
    serial: &mut S,
    timer: &mut T,
    timeout: T::Time,
) -> Result<u8, Error<S::Error>>
where
    T: hal::timer::CountDown,
    S: hal::serial::Read<u8>,
{
    timer.start(timeout);

    loop {
        match serial.read() {
            // raise error
            Err(nb::Error::Other(e)) => return Err(Error::Serial(e)),
            Err(nb::Error::WouldBlock) => {
                // no data available yet, check the timer below
            },
            Ok(byte) => return Ok(byte),
        }

        match timer.wait() {
            Err(nb::Error::Other(_e)) => {
                // The error type specified by `timer.wait()` is `!`, which
                // means no error can actually occur. The Rust compiler
                // still forces us to provide this match arm, though.
                unreachable!()
            },
            // no timeout yet, try again
            Err(nb::Error::WouldBlock) => continue,
            Ok(()) => return Err(Error::TimedOut),
        }
    }
}

pub fn write_all<S>(serial: &mut S, buffer: &[u8]) -> Result<(), S::Error>
where
    S: hal::serial::Write<u8>
{
    for &byte in buffer {
        block!(serial.write(byte))?;
    }

    Ok(())
}

