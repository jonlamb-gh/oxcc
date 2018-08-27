use board::DacSampleAverageCount;
use num;
use ranges::{self, Bounded, BoundedSummation, BoundedConstDiv};
use typenum::consts::*;

type U4095 = op! { U4096 - U1 };
pub type AdcInput = Bounded<u16, U0, U4095>;

pub struct DualSignal<T: HighLowReader> {
    high: AdcInput,
    low: AdcInput,
    reader: T,
}

impl<T> DualSignal<T>
where
    T: HighLowReader,
{
    pub fn new(high: AdcInput, low: AdcInput, high_low_reader: T) -> Self {
        DualSignal {
            high,
            low,
            reader: high_low_reader,
        }
    }

    pub fn update(&mut self) {
        self.high = self.reader.read_high();
        self.low = self.reader.read_low();
    }

    // not sure if the averaging is needed, we might be able to just use a
    // single read with large Cycles480 sample time?
    // https://github.com/jonlamb-gh/oscc/blob/devel/firmware/common/libs/dac/oscc_dac.cpp#L17
    pub fn prevent_signal_discontinuity(&mut self) {
        let low_sum = ranges::Summation::<u32, U0, DacSampleAverageCount>::eval(|_| self.reader.read_low());
        let high_sum = ranges::Summation::<u32, U0, DacSampleAverageCount>::eval(|_| self.reader.read_high());

        self.low = ranges::retype(ranges::coerce(ranges::ConstDiv::<DacSampleAverageCount>::eval(low_sum)));
        self.high = ranges::retype(ranges::coerce(ranges::ConstDiv::<DacSampleAverageCount>::eval(high_sum)));
    }

    pub fn average(&self) -> AdcInput {
        let low: u16 = self.low.val();
        let high: u16 = self.high.val();
        AdcInput::clamp(((u32::from(high) + u32::from(low)) / 2) as u16)
    }

    pub fn diff(&self) -> u16 {
        let low: u16 = self.low.val();
        let high: u16 = self.high.val();
        num::abs(i32::from(high) - i32::from(low)) as u16
    }

    pub fn high(&self) -> AdcInput {
        self.high
    }

    pub fn low(&self) -> AdcInput {
        self.low
    }
}

pub trait HighLowReader {
    fn read_high(&self) -> AdcInput;
    fn read_low(&self) -> AdcInput;
}
