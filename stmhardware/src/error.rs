use types;
use esp8266;
use dhtxx;

/**
  Wrapper for esp8266 errors which can't implement `Fail` because that would
  force us to be more specific about type parameters
*/
#[derive(Fail, Debug)]
#[fail(display = "{:?}", err)]
pub struct EspError {
    err: esp8266::Error<types::SerialReadError, types::SerialWriteError>
}
#[derive(Fail, Debug)]
#[fail(display = "{:?}", err)]
pub struct EspTransmissionError {
    err: esp8266::TransmissionError<types::SerialReadError, types::SerialWriteError>
}
impl From<esp8266::TransmissionError<types::SerialReadError, types::SerialWriteError>> for EspTransmissionError {
    fn from(err: esp8266::TransmissionError<types::SerialReadError, types::SerialWriteError>) -> Self {
        Self{err}
    }
}

////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Esp8266 error: {:?}", _0)]
    EspError (#[cause] EspError),
    #[fail(display = "Esp8266 error: {:?}", _0)]
    Esp8266TransmissionError (#[cause] EspTransmissionError),
    #[fail(display = "Dhtxx error: {}", _0)]
    DhtxxError ( #[cause] dhtxx::Error),
    #[fail(display = "Capacity error: {:?}", err)]
    CapacityError {
        err: ::arrayvec::CapacityError,
    },
    #[fail(display = "Formatting error: {:?}", err)]
    FmtError {
        err: ::core::fmt::Error,
    }
}

// This might not be the best way to do this but I don't believe that the Error
// type is available in #[no_std]. Also, trying to use the #[cause] attribute forces
// the other error to impl Fail which requires additional bounds on them
impl<T> From<::arrayvec::CapacityError<T>> for Error {
    fn from(other: ::arrayvec::CapacityError<T>) -> Self {
        Error::CapacityError{err: other.simplify()}
    }
}
impl From<::core::fmt::Error> for Error {
    fn from(other: ::core::fmt::Error) -> Self {
        Error::FmtError{err: other}
    }
}
