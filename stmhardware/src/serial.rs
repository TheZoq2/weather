extern crate embedded_hal as hal;
extern crate nb;

use hal::prelude::*;
use stm32f30x_hal::stm32f30x;

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

/**
  Reads bytes from the serial into `buffer`. The last if more than `buffer.len()` bytes
  that are read, the last `buffer.len()` bytes are stoed.

  Returns Ok(n) where n is the amount of bytes read into buffer
*/
pub fn read_until_timeout<S, T, T::Time>(
    tx: &mut S,
    timer: &mut T,
    timeout: T::Time
    buffer: &mut [u8]
) -> Result<usize, Error<S::Error>>
where
    T: hal::timer::CountDown,
    S: hal::serial::Read<u8>
{
    let mut ptr = 0;
    let mut byte_amount = 0;
    loop {
        match serial::read_with_timeout(&mut rx, &mut timer, Hertz(1)) {
            Ok(byte) => {
                buffer[ptr] = byte;
                ptr += 1;
                ptr = ptr % buffer.len();
                bytes_received = true;
            },
            Err(serial::Error::TimedOut) => {
                if bytes_received {
                    break;
                }
                else {
                    continue;
                }
            },
            Err(_e) => {
                panic!()
            }
        };
    }
}

pub fn clear_isr(usart: stm32f30x::USART1) {
    usart.icr.write(|w| {
        w.orecf().set_bit()
    })
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

