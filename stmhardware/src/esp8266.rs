extern crate embedded_hal as hal;
extern crate nb;

use cortex_m::asm;

use embedded_hal_time::{RealCountDown, Millisecond, Second, Microsecond};
use core::cmp::min;
use core::fmt::{self, Write};
use arrayvec::{CapacityError, ArrayString};
use itoa;

// use hal::prelude::*;


use serial;

/**
    Maximum length of an AT response (Length of message + CRLF)

    longest message: `WIFI GOT IP\r\n`
*/
const AT_RESPONSE_BUFFER_SIZE: usize = 13;

/**
  Possible responses from an esp8266 AT command
*/
#[derive(Debug, PartialEq)]
pub enum ATResponse {
    Ok,
    Error,
    Busy,
    WiFiGotIp,
}

/**
  Error type for esp communication.

  `R` and `T` are the error types of the serial module
*/
#[derive(Debug)]
pub enum Error<R, T> {
    /// Serial transmission errors
    TxError(T),
    /// Serial reception errors
    RxError(R),
    /// Invalid or unexpected data received from the device
    UnexpectedResponse(ATResponse),
    /// Errors from the formating of messages
    Fmt(fmt::Error),
    /// Error indicating an ArrayString wasn't big enough
    Capacity(CapacityError)
}
impl<R,T> From<fmt::Error> for Error<R,T>{
    fn from(other: fmt::Error) -> Error<R,T> {
        Error::Fmt(other)
    }
}
impl<R,T,ErrType> From<CapacityError<ErrType>> for Error<R,T>{
    fn from(other: CapacityError<ErrType>) -> Error<R,T> {
        Error::Capacity(other.simplify())
    }
}

/**
    Indicates what step in the data transmission that the sensor is in. Used
    in `TransmissionError` for reporting information about where things went wrong
*/
#[derive(Debug)]
pub enum TransmissionStep {
    Connect,
    Send,
    Close
}
#[derive(Debug)]
pub struct TransmissionError<R, T> {
    step: TransmissionStep,
    cause: Error<R, T>
}

impl<R, T> TransmissionError<R, T> {
    pub fn try<RetType>(step: TransmissionStep, cause: Result<RetType, Error<R, T>>) 
        -> Result<RetType, Self>
    {
        cause.map_err(|e| {
            Self {
                step,
                cause: e
            }
        })
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

macro_rules! transmission_return_type {
    ($ok:ty) => {
        Result<$ok, TransmissionError<serial::Error<Rx::Error>, Tx::Error>>
    }
}


////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////


/**
  Struct for interracting with an esp8266 wifi module over USART
*/
pub struct Esp8266<Tx, Rx, Timer, Rst>
where Tx: hal::serial::Write<u8>,
      Rx: hal::serial::Read<u8>,
      Timer: RealCountDown<Millisecond>,
      Timer: RealCountDown<Microsecond>,
      Timer: hal::timer::CountDown,
      Timer::Time: Copy,
      Rst: hal::digital::OutputPin
{
    tx: Tx,
    rx: Rx,
    timer: Timer,
    timeout: Millisecond,
    reset_pin: Rst
}

impl<Tx, Rx, Timer, Rst> Esp8266<Tx, Rx, Timer, Rst>
where Tx: hal::serial::Write<u8>,
      Rx: hal::serial::Read<u8>,
      Timer: RealCountDown<Millisecond>,
      Timer: RealCountDown<Microsecond>,
      Timer: hal::timer::CountDown,
      Timer::Time: Copy,
      Rst: hal::digital::OutputPin
{

    /**
      Sets up the esp8266 struct and configures the device for future use

      `tx` and `rx` are the pins used for serial communication, `timer` is
      a hardware timer for dealing with things like serial timeout and
      `reset_pin` is a pin which must be connected to the reset pin
      of the device
    */
    pub fn new(tx: Tx, rx: Rx, timer: Timer, reset_pin: Rst)
        -> return_type!(Self)
    {
        let timeout = Millisecond(5000);
        let mut result = Self {tx, rx, timer, timeout, reset_pin};

        result.reset()?;

        // Turn off echo on the device and wait for it to process that command
        result.send_at_command("E0")?;

        // Make sure we got an OK from the esp
        result.wait_for_ok()?;

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
        address: &str,
        port: u16,
        data: &str
    ) -> transmission_return_type!(())
    {
        // Send a start connection message
        let tcp_start_result = self.start_tcp_connection(connection_type, address, port);
        TransmissionError::try(TransmissionStep::Connect, tcp_start_result)?;

        TransmissionError::try(TransmissionStep::Send, self.transmit_data(data))?;

        TransmissionError::try(TransmissionStep::Close, self.close_connection())
    }

    pub fn close_connection(&mut self) -> return_type!(()) {
        self.send_at_command("+CIPCLOSE")?;
        self.wait_for_ok()
    }

    /*
    /**
        Puts the sensor into sleep mode. Due to the way that the esp-01 handles
        sleep mode, the whole sensor will sleep for `time_millis` milliseconds.

        After that time, some parts of the sensor will wake up and consume more power,
        however, it will not be operational until its reset pin is pulled low for a short
        while. This can be done using the `wake_up` method

        For more information, see https://tzapu.com/minimalist-battery-powered-esp8266-wifi-temperature-logger/
    */
    pub fn enter_sleep_mode(&mut self, time_millis: u32) -> return_type!(()) {
        // Maximum length of the `time_millis` value
        const MILLIS_LENGTH: usize = 10;

        const MSG_LENGTH: usize = 8 + MILLIS_LENGTH;
        let mut msg_str = ArrayString::<[_;MSG_LENGTH]>::from("AT+GSLP=")?;

        let mut millis_str = ArrayString::<[_; MILLIS_LENGTH]>::new();
        itoa::fmt(&mut millis_str, time_millis)?;

        msg_str.try_push_str(&millis_str)?;
        self.send_at_command(&msg_str)
    }
    */

    pub fn reset(&mut self) -> return_type!(()) {
        self.reset_pin.set_low();

        // Give the esp some time to react
        self.timer.start_real(Millisecond(10));
        // This unwrap is safe because wait() returns `Result<(), Void>`
        block!(self.timer.wait()).unwrap();

        self.reset_pin.set_high();

        // Because the device sends some random incorrect data after/while being reset
        // we need to read some data until we get valid data again
        loop {
            let result = serial::read_with_timeout(
                &mut self.rx,
                &mut self.timer,
                Millisecond(1000)
            );

            if let Ok(byte) = result {
                break;
            }
        }

        self.wait_for_got_ip()
    }

    fn transmit_data(&mut self, data: &str) -> return_type!(()) {
        self.start_transmission(data.len())?;
        self.wait_for_prompt()?;
        self.send_raw(data.as_bytes())?;
        self.wait_for_ok()
    }

    fn start_tcp_connection (
        &mut self,
        connection_type: ConnectionType,
        address: &str,
        port: u16
    ) -> return_type!(())
    {
        // Length of biggest u16:
        const PORT_STRING_LENGTH: usize = 5;
        let mut port_str = ArrayString::<[_;PORT_STRING_LENGTH]>::new();
        // write!(&mut port_str, "{}", port)?;
        itoa::fmt(&mut port_str, port)?;

        self.send_raw("AT+CIPSTART=\"".as_bytes())?;
        self.send_raw(connection_type.as_str().as_bytes())?;
        self.send_raw("\",\"".as_bytes())?;
        self.send_raw(address.as_bytes())?;
        self.send_raw("\",".as_bytes())?;
        self.send_raw(port_str.as_bytes())?;
        self.send_raw("\r\n".as_bytes())?;
        self.wait_for_ok()
    }

    fn start_transmission(&mut self, message_length: usize) -> return_type!(()) {
        // You can only send 2048 bytes per packet 
        assert!(message_length < 2048);
        let mut length_buffer = ArrayString::<[_; 4]>::new();
        // write!(&mut length_buffer, "{}", message_length)?;
        itoa::fmt(&mut length_buffer, message_length)?;

        self.send_raw(b"AT+CIPSEND=")?;
        self.send_raw(length_buffer.as_bytes())?;
        self.send_raw(b"\r\n")?;
        Ok(())
    }

    /**
      Sends the "AT${command}" to the device
    */
    fn send_at_command(&mut self, command: &str) -> return_type!(()) {
        self.send_raw(b"AT")?;
        self.send_raw(command.as_bytes())?;
        self.send_raw(b"\r\n")?;
        Ok(())
    }

    fn wait_for_at_response(&mut self, expected_response: &ATResponse) -> return_type!(()) {
        let mut buffer = [0; AT_RESPONSE_BUFFER_SIZE];
        let response = serial::read_until_message(
            &mut self.rx,
            &mut self.timer,
            self.timeout,
            &mut buffer,
            &parse_at_response
        );

        match response {
            Ok(ref resp) if resp == expected_response => {
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

    fn wait_for_ok(&mut self) -> return_type!(()) {
        self.wait_for_at_response(&ATResponse::Ok)
    }
    fn wait_for_got_ip(&mut self) -> return_type!(()) {
        self.wait_for_at_response(&ATResponse::WiFiGotIp)
    }

    fn wait_for_prompt(&mut self) -> return_type!(()) {
        let mut buffer = [0; 1];
        let result = serial::read_until_message(
            &mut self.rx,
            &mut self.timer,
            self.timeout,
            &mut buffer,
            &|buf, ptr| {
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

    fn send_raw(&mut self, bytes: &[u8]) -> return_type!(()) {
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
    else if compare_circular_buffer(buffer, offset, "busy p...\r\n".as_bytes()) {
        Some(ATResponse::Busy)
    }
    else if compare_circular_buffer(buffer, offset, "WIFI GOT IP\r\n".as_bytes()) {
        Some(ATResponse::WiFiGotIp)
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

