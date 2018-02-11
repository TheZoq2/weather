extern crate embedded_hal as hal;
extern crate nb;
use core::cmp::min;

use hal::prelude::*;


use serial;

// Maximum length of an AT response (Length of message + CRLF
const AT_RESPONSE_BUFFER_SIZE: usize = 13;

pub enum ATResponse {
    Ok,
    Connect,
    Ring,
    NoCarrier,
    Error,
    NoDialtone,
    Busy,
    NoAnswer
}

pub fn send_at_command<S>(serial: &mut S, command: &str) -> Result<(), S::Error> 
where
    S: hal::serial::Write<u8>
{
    serial::write_all(serial, "AT+".as_bytes())?;
    serial::write_all(serial, command.as_bytes())?;
    serial::write_all(serial, "\r\n".as_bytes())?;
    Ok(())
}

/**
  Tries to read an AT response from the serial port.

  `timeout` is a function to get around borrowck
*/
pub fn read_at_response<T, S, F>(
    serial: &mut S,
    timer: &mut T,
    timeout: F
) -> Result<ATResponse, serial::Error<S::Error>>
where
    T: hal::timer::CountDown,
    S: hal::serial::Read<u8>,
    F: Fn() -> T::Time
{
    let mut buffer = [0; AT_RESPONSE_BUFFER_SIZE];

    loop {
        let new_byte = serial::read_with_timeout(serial, timer, timeout())?;

        // Shift the previous content one step
        for i in 1..buffer.len() {
            buffer[i] = buffer[i-1];
        }
        buffer[0] = new_byte;

        let at_response = parse_at_response(&buffer);
        if let Some(at_response) = at_response {
            return Ok(at_response)
        }
    }
}


/**
  Parses `buffer` as an AT command response returning the type if it
  is a valid AT response and `None` otherwise
*/
fn parse_at_response(buffer: &[u8]) -> Option<ATResponse> {
    if compare_buffers(buffer, "OK\r\n".as_bytes()) {
        Some(ATResponse::Ok)
    }
    else if compare_buffers(buffer, "CONNECT\r\n".as_bytes()) {
        Some(ATResponse::Connect)
    }
    else if compare_buffers(buffer, "RING\r\n".as_bytes()) {
        Some(ATResponse::Ring)
    }
    else if compare_buffers(buffer, "NO CARRIER\r\n".as_bytes()) {
        Some(ATResponse::NoCarrier)
    }
    else if compare_buffers(buffer, "ERROR\r\n".as_bytes()) {
        Some(ATResponse::Error)
    }
    else if compare_buffers(buffer, "NO DIALTONE\r\n".as_bytes()) {
        Some(ATResponse::NoDialtone)
    }
    else if compare_buffers(buffer, "BUSY\r\n".as_bytes()) {
        Some(ATResponse::Busy)
    }
    else if compare_buffers(buffer, "NO ANSWER\r\n".as_bytes()) {
        Some(ATResponse::NoAnswer)
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
        if buffer1[i] != buffer2[i] {
            return false;
        }
    }
    return true;
}
