#![no_std]

extern crate cortex_m;
extern crate stm32f7;

#[allow(dead_code, unused_variables)]
mod bootload;
#[allow(dead_code, unused_variables)]
mod config;

use stm32f7::stm32f7x7;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Success,
    InvalidAddress,
    LengthNotMultiple4,
    LengthTooLong,
    DataLengthIncorrect,
    EraseError,
    WriteError,
    FlashError,
    NetworkError,
    InternalError,
}

pub type Result<T> = core::result::Result<T, Error>;

pub use self::bootload::reset_to_bootloader;
