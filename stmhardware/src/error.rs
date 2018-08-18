use types;
use esp8266;
use dhtxx;

#[derive(Debug, Fail)]
enum Error {
    #[fail(display = "Esp8266 error: {:?}", err)]
    Esp8266Error {
        err: esp8266::Error<types::SerialReadError, types::SerialWriteError>,
    },

    #[fail(display = "Dhtxx error: {:?}", err)]
    DhtxxError {
        err: dhtxx::Error,
    }
}
