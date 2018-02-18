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
pub fn read_until_timeout<S, T, F>(
    rx: &mut S,
    timer: &mut T,
    timeout: &F,
    buffer: &mut [u8],
) -> Result<usize, Error<S::Error>>
where
    T: hal::timer::CountDown,
    S: hal::serial::Read<u8>,
    F: Fn() -> T::Time,
{
    let mut ptr = 0;
    let mut byte_amount = 0;
    loop {
        match read_with_timeout(rx, timer, timeout()) {
            Ok(byte) => {
                buffer[ptr] = byte;
                ptr = (ptr+1) % buffer.len();
                byte_amount += 1;
            },
            Err(Error::TimedOut) => {
                // If the remote end has already sent bytes and has now
                // stopped, we assume the transmission has ended
                if byte_amount != 0 {
                    break;
                }
                else {
                    continue;
                }
            },
            Err(e) => {
                return Err(e)
            }
        };
    }

    if byte_amount >= buffer.len() {
        uncircularize(buffer, ptr);
        Ok(buffer.len())
    }
    else {
        Ok(ptr)
    }
}

fn uncircularize(buf: &mut [u8], offset: usize) {
    for i in 0..offset {
        let first = buf[0];
        for n in 0..buf.len()-1 {
            buf[n] = buf[n+1];
        }
        buf[buf.len()-1] = first;
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

