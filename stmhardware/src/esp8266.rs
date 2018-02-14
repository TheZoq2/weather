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
    serial::write_all(serial, "AT".as_bytes())?;
    serial::write_all(serial, command.as_bytes())?;
    serial::write_all(serial, "\r\n".as_bytes())?;
    Ok(())
}

pub fn wait_for_at_reply<S, T, F>(
    rx: &mut S,
    timer: &mut T,
    timeout: F,
) -> Result<ATResponse, serial::Error<S::Error>>
where
    S: hal::serial::Read<u8>,
    T: hal::timer::CountDown,
    F: Fn() -> T::Time
{
    // Setup a buffer to read the data into
    let mut buffer = ATResponseBuffer::new();

    // Read data until the serial port times out
    let mut has_timed_out = false;
    while !has_timed_out {
        match serial::read_with_timeout(rx, timer, timeout()) {
            Ok(byte) => {
                buffer.push_byte(byte)
            },
            Err(serial::Error::TimedOut) => {
                has_timed_out = true;
                break;
            }
            Err(e) => {
                Err(e)?
            }
        }
    }

    match buffer.parse() {
        Some(response) => Ok(response),
        None => Err(serial::Error::TimedOut)
    }
}


pub struct ATResponseBuffer {
    buffer: [u8; AT_RESPONSE_BUFFER_SIZE]
}

impl ATResponseBuffer {
    pub fn new() -> Self {
        Self {
            buffer: [0; AT_RESPONSE_BUFFER_SIZE]
        }
    }

    pub fn push_byte(&mut self, byte: u8) {
        // Shift the previous content one step
        for i in 1..self.buffer.len() {
            self.buffer[i] = self.buffer[i-1];
        }
        self.buffer[0] = byte;
    }

    pub fn parse(&self) -> Option<ATResponse> {
        parse_at_response(&self.buffer)
    }
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
