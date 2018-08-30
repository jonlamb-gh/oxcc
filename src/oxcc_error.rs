// might make more sense to just use the existing HAL errors?

use nucleo_f767zi::hal::can::CanError;
use nucleo_f767zi::hal::spi;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OxccError {
    IO,
    CanBusTxTimeout,
}

impl From<spi::Error> for OxccError {
    fn from(e: spi::Error) -> OxccError {
        match e {
            _ => OxccError::IO,
        }
    }
}

impl From<CanError> for OxccError {
    fn from(e: CanError) -> OxccError {
        match e {
            CanError::Timeout => OxccError::CanBusTxTimeout,
            _ => OxccError::IO,
        }
    }
}
