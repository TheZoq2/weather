extern crate embedded_hal as hal;
extern crate nb;
use core::cmp::min;

use hal::prelude::*;


use serial;

// Maximum length of an AT response (Length of message + CRLF
const AT_RESPONSE_BUFFER_SIZE: usize = 13;

#[derive(Debug)]
pub enum ATResponse {
    Ok,
    Connect,
    Error,
    Busy,
}

pub fn send_at_command<S>(serial: &mut S, command: &str) -> Result<(), S::Error> 
where
    S: hal::serial::Write<u8>
{
    serial::write_all(serial, "AT".as_bytes())?;
    serial::write_all(serial, command.as_bytes())?;
    serial::write_all(serial, "\r\n".as_bytes())?;
    Ok(())
}

pub fn wait_for_at_reply<S, T, F>(
    rx: &mut S,
    timer: &mut T,
    timeout: &F,
) -> Result<Option<ATResponse>, serial::Error<S::Error>>
where
    S: hal::serial::Read<u8>,
    T: hal::timer::CountDown,
    F: Fn() -> T::Time
{
    let mut buffer = [0; AT_RESPONSE_BUFFER_SIZE];
    let reply_length = serial::read_until_timeout(rx, timer, timeout, &mut buffer)?;

    // Flip the buffer around to make it easer to find the last bytes of the message
    let mut flipped_buffer = [0; AT_RESPONSE_BUFFER_SIZE];
    for i in 0..buffer.len() {
        flipped_buffer[buffer.len()-i-1] = buffer[i];
    }

    Ok(parse_at_response(&flipped_buffer))
}

/**
  Parses `buffer` as an AT command response returning the type if it
  is a valid AT response and `None` otherwise
*/
pub fn parse_at_response(buffer: &[u8]) -> Option<ATResponse> {
    if compare_buffers(buffer, "OK\r\n".as_bytes()) {
        Some(ATResponse::Ok)
    }
    else if compare_buffers(buffer, "CONNECT\r\n".as_bytes()) {
        Some(ATResponse::Connect)
    }
    else if compare_buffers(buffer, "ERROR\r\n".as_bytes()) {
        Some(ATResponse::Error)
    }
    else if compare_buffers(buffer, "BUSY\r\n".as_bytes()) {
        Some(ATResponse::Busy)
    }
    else {
        None
    }
}

/**
  Compares the content of the two specified buffers. If the first `n` bytes
  match, true is returned. `n` is the minimum length of the two buffers.
*/
fn compare_buffers(buffer1: &[u8], buffer2: &[u8]) -> bool {
    for i in 0..min(buffer1.len(), buffer2.len()) {
        if buffer1[i] != buffer2[buffer2.len()-1-i] {
            return false;
        }
    }
    return true;
}
