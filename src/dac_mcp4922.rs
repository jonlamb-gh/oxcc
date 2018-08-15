// TODO
// - latching
// - gain
// - buffer vref

use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::{Mode, Phase, Polarity};

/// SPI mode
pub const MODE: Mode = Mode {
    phase: Phase::CaptureOnFirstTransition,
    polarity: Polarity::IdleLow,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Channel {
    ChannelA,
    ChannelB,
}

pub struct Mcp4922<SPI, CS> {
    spi: SPI,
    cs: CS,
}

impl<SPI, CS, E> Mcp4922<SPI, CS>
where
    SPI: Transfer<u8, Error = E>,
    CS: OutputPin,
{
    pub fn new(spi: SPI, mut cs: CS) -> Self {
        // unselect the device
        cs.set_high();

        Mcp4922 { spi, cs }
    }

    pub fn output_ab(&mut self, output_a: u16, output_b: u16) {
        // TODO latching?
        self.output(output_a, Channel::ChannelA);
        self.output(output_b, Channel::ChannelB);
    }

    pub fn output(&mut self, data: u16, channel: Channel) {
        self.cs.set_low();

        let mut buffer = [0u8; 2];
        // bits 11 through 0: data
        buffer[0] = (data & 0x00FF) as _;
        buffer[1] = 0x00
            | ((data >> 8) & 0x000F) as u8
            // bit 12: shutdown bit. 1 for active operation
            | (1 << 4)
            // bit 13: gain bit; 0 for 1x gain, 1 for 2x
            // bit 14: buffer VREF?
            // bit 15: 0 for DAC A, 1 for DAC B
            | u8::from(channel) << 7;

        if let Err(_) = self.spi.transfer(&mut buffer) {
            // TODO - error handling
        }

        self.cs.set_high();
    }
}

impl From<Channel> for u8 {
    fn from(c: Channel) -> u8 {
        match c {
            Channel::ChannelA => 0b0,
            Channel::ChannelB => 0b1,
        }
    }
}
