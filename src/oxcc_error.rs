// might make more sense to just use the existing HAL errors?

use nucleo_f767zi::hal::can::CanError;
use nucleo_f767zi::hal::spi;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OxccError {
    Spi(spi::Error),
    Can(CanError),
}

impl From<spi::Error> for OxccError {
    fn from(e: spi::Error) -> Self {
        OxccError::Spi(e)
    }
}

impl From<CanError> for OxccError {
    fn from(e: CanError) -> Self {
        OxccError::Can(e)
    }
}
