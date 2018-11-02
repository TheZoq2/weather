use types;
use esp8266;
use dhtxx;

macro_rules! ctx_error {
    ($name:ident {$($cause_name:ident($cause:path)),*}) => {
        #[derive(Debug)]
        pub enum $name {
            $(
                $cause_name($cause)
            ),*
        }
        $(
            impl From<$cause> for $name {
                fn from(other: $cause) -> Self {
                    $name::$cause_name(other)
                }
            }
        )*
    }

}

pub type EspError = esp8266::Error<types::SerialReadError, types::SerialWriteError>;
pub type EspTransmissionError = esp8266::TransmissionError<types::SerialReadError, types::SerialWriteError>;


ctx_error!(AnemometerError {
    EspTransmission(EspTransmissionError),
    Encoding(EncodingError)
});

ctx_error!(DhtError {
    EspTransmission(EspTransmissionError),
    Encoding(EncodingError),
    Dht(dhtxx::Error)
});


ctx_error!(EncodingError {
    Fmt(::core::fmt::Error),
    CapacityError(::arrayvec::CapacityError)
});

ctx_error!(BatteryReadError {
    Encoding(EncodingError),
    EspTransmission(EspTransmissionError)
});

ctx_error!(ReadLoopError {
    Anemometer(AnemometerError),
    Dht(DhtError),
    EspError(EspError),
    Battery(BatteryReadError)
});

