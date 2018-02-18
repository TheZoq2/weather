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
) -> Result<ATResponse, serial::Error<S::Error>>
where
    S: hal::serial::Read<u8>,
    T: hal::timer::CountDown,
    F: Fn() -> T::Time
{
    let mut buffer = [0; AT_RESPONSE_BUFFER_SIZE];
    serial::read_until_message(
        rx,
        timer,
        timeout,
        &mut buffer,
        &parse_at_response
    )
}

/**
  Parses `buffer` as an AT command response returning the type if it
  is a valid AT response and `None` otherwise
*/
pub fn parse_at_response(buffer: &[u8], offset: usize) -> Option<ATResponse> {
    if compare_circular_buffer(buffer, offset, "OK\r\n".as_bytes()) {
        Some(ATResponse::Ok)
    }
    else if compare_circular_buffer(buffer, offset, "ERROR\r\n".as_bytes()) {
        Some(ATResponse::Error)
    }
    else if compare_circular_buffer(buffer, offset, "BUSY\r\n".as_bytes()) {
        Some(ATResponse::Busy)
    }
    else {
        None
    }
}

/**
  Compares the content of a circular buffer with another buffer. The comparison
  is done 'from the back' and if one buffer is longer than the other, only the
  content of the shared bytes is compared.

  `offset` is the index of the first byte of the circular buffer
  ```
  [4,5,0,1,2,3]
       ^- offset
  ```
*/
pub fn compare_circular_buffer(
    circular_buffer: &[u8],
    offset: usize,
    comparison: &[u8]
) -> bool
{
    let comparison_length = min(circular_buffer.len(), comparison.len());
    for i in 0..comparison_length {
        // Addition of circular_buffer.len() because % is remainder, not mathematical modulo
        // https://stackoverflow.com/questions/31210357/is-there-a-modulus-not-remainder-function-operation/31210691
        let circular_index = (circular_buffer.len() + offset - 1 - i) % circular_buffer.len();
        let comparison_index = comparison.len() - 1 - i;
        if circular_buffer[circular_index] != comparison[comparison_index] {
            return false;
        }
    }
    true
}

