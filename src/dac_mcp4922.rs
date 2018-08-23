// TODO
// - latching
// - gain
// - buffer vref

use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::{Mode, Phase, Polarity};

use ranges::Bounded;
use typenum::{U0, U1, U4096};

type U4095 = op! { U4096 - U1 };

/// It's a 12 bit dac, so the upper bound is 4095 (2^12 - 1)
pub type DacOutput = Bounded<u16, U0, U4095>;

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

    pub fn output_ab(&mut self, output_a: DacOutput, output_b: DacOutput) {
        // TODO latching?
        self.output(output_a, Channel::ChannelA);
        self.output(output_b, Channel::ChannelB);
    }

    pub fn output(&mut self, data: DacOutput, channel: Channel) {
        self.cs.set_low();

        let mut buffer = [0u8; 2];
        // bits 11 through 0: data
        buffer[0] = (data.val() & 0x00FF) as _;
        buffer[1] = ((data.val() >> 8) & (0x000F as u16)) as u8
            // bit 12: shutdown bit. 1 for active operation
            | (1 << 4)
            // bit 13: gain bit; 0 for 1x gain, 1 for 2x
            // bit 14: buffer VREF?
            // bit 15: 0 for DAC A, 1 for DAC B
            | u8::from(channel) << 7;

        if self.spi.transfer(&mut buffer).is_err() {
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
