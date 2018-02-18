extern crate embedded_hal as hal;
extern crate nb;
use core::cmp::min;
use core::fmt::{self, Write};
use arrayvec::ArrayString;

// use hal::prelude::*;


use serial;

// Maximum length of an AT response (Length of message + CRLF
const AT_RESPONSE_BUFFER_SIZE: usize = 13;

#[derive(Debug)]
pub enum ATResponse {
    Ok,
    Error,
    Busy,
}

#[derive(Debug)]
pub enum Error<R, T> {
    TxError(T),
    RxError(R),
    UnexpectedResponse(ATResponse),
    Fmt(fmt::Error)
}

impl<R,T> From<fmt::Error> for Error<R,T>{
    fn from(other: fmt::Error) -> Error<R,T> {
        Error::Fmt(other)
    }
}

pub enum ConnectionType {
    Tcp,
    Udp
}
impl ConnectionType {
    pub fn as_str(&self) -> &str {
        match *self {
            ConnectionType::Tcp => "TCP",
            ConnectionType::Udp => "UDP"
        }
    }
}


macro_rules! return_type {
    ($ok:ty) => {
        Result<$ok, Error<serial::Error<Rx::Error>, Tx::Error>>
    }
}

/**
  Struct for interracting with an esp8266 wifi module over USART
*/
pub struct Esp8266<Tx, Rx, Timer, Timeout>
where Tx: hal::serial::Write<u8>,
      Rx: hal::serial::Read<u8>,
      Timer: hal::timer::CountDown,
      Timeout: Fn() -> Timer::Time
{
    tx: Tx,
    rx: Rx,
    timer: Timer,
    timeout: Timeout
}

impl<Tx, Rx, Timer, Timeout> Esp8266<Tx, Rx, Timer, Timeout>
where Tx: hal::serial::Write<u8>,
      Rx: hal::serial::Read<u8>,
      Timer: hal::timer::CountDown,
      Timeout: Fn() -> Timer::Time
{
    pub fn new(tx: Tx, rx: Rx, timer: Timer, timeout: Timeout) -> return_type!(Self)
    {
        let mut result = Self {tx, rx, timer, timeout};

        // Turn off echo on the device and wait for it to process that command
        result.send_at_command("E0")?;

        for _ in 0..3 {
            result.timer.start((result.timeout)());
            block!(result.timer.wait()).unwrap();
        }
        // Read a byte, ignore the result. This is done to clear the buffer
        let _byte = result.wait_for_ok();

        Ok(result)
    }

    /**
      Sends `AT${command}` to the device and waits for it to reply
    */
    pub fn communicate(&mut self, command: &str) -> return_type!(())
    {
        self.send_at_command(command)?;
        self.wait_for_ok()
    }

    pub fn send_data(
        &mut self,
        connection_type: ConnectionType,
        address: &[u8],
        port: &[u8],
        data: &[u8]
    ) -> return_type!(())
    {
        // Send a start connection message
        self.start_tcp_connection(connection_type, address, port)?;

        self.wait_for_ok()?;
        self.start_transmission(data.len())?;
        //wait_for_prompt()
        unimplemented!("Send data");
        unimplemented!("Close connection");
    }

    fn start_tcp_connection (
        &mut self,
        connection_type: ConnectionType,
        address: &[u8],
        port: &[u8]
    ) -> return_type!(())
    {
        self.send_raw("AT+CIPSTART=\"".as_bytes())?;
        self.send_raw(connection_type.as_str().as_bytes())?;
        self.send_raw("\",\"".as_bytes())?;
        self.send_raw(address)?;
        self.send_raw("\",".as_bytes())?;
        self.send_raw(port)?;
        self.send_raw("\r\n".as_bytes())?;
        Ok(())
    }

    fn start_transmission(&mut self, message_length: usize) -> return_type!(())
    {
        // You can only send 2048 bytes per packet 
        assert!(message_length < 2048);
        let mut length_buffer = ArrayString::<[_; 4]>::new();
        write!(&mut length_buffer, "{}", message_length)?;

        self.send_raw(b"AT+CIPSEND=")?;
        self.send_raw(length_buffer.as_bytes())?;
        self.send_raw(b"\r\n")?;
        Ok(())
    }

    /**
      Sends the "AT${command}" to the device
    */
    fn send_at_command(&mut self, command: &str) -> return_type!(())
    {
        self.send_raw(b"AT")?;
        self.send_raw(command.as_bytes())?;
        self.send_raw(b"\r\n")?;
        Ok(())
    }

    fn wait_for_ok(&mut self) -> return_type!(()) {
        let mut buffer = [0; AT_RESPONSE_BUFFER_SIZE];
        let response = serial::read_until_message(
            &mut self.rx,
            &mut self.timer,
            &self.timeout,
            &mut buffer,
            &parse_at_response
        );

        match response {
            Ok(ATResponse::Ok) => {
                Ok(())
            },
            Ok(other) => {
                Err(Error::UnexpectedResponse(other))
            }
            Err(e) => {
                Err(Error::RxError(e))
            }
        }
    }

    fn wait_for_prompt(&mut self) -> return_type!(()) {
        let mut buffer = [0; 1];
        let result = serial::read_until_message(
            &mut self.rx,
            &mut self.timer,
            &self.timeout,
            &mut buffer,
            &|buf, _| {
                if buf[0] == '>' as u8 {
                    Some(())
                }
                else {
                    None
                }
            }
        );
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::RxError(e))
        }
    }

    fn send_raw(&mut self, bytes: &[u8]) -> return_type!(())
    {
        match serial::write_all(&mut self.tx, bytes) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::TxError(e))
        }
    }
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

