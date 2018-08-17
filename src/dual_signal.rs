use board::{DAC_SAMPLE_AVERAGE_COUNT};
use num;

pub struct DualSignal<T: HighLowReader> {
    high: u16,
    low: u16,
    reader: T
}

impl <T> DualSignal<T> where T: HighLowReader {
    pub fn new(high: u16, low: u16, high_low_reader: T) -> Self {
        DualSignal {
            high,
            low,
            reader: high_low_reader
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
        let mut low: u32 = 0;
        let mut high: u32 = 0;

        for _ in 0..DAC_SAMPLE_AVERAGE_COUNT {
            low += self.reader.read_low() as u32;
        }

        for _ in 0..DAC_SAMPLE_AVERAGE_COUNT {
            high += self.reader.read_high() as u32;
        }

        self.low = (low / DAC_SAMPLE_AVERAGE_COUNT) as _;
        self.high = (high / DAC_SAMPLE_AVERAGE_COUNT) as _;
    }

    pub fn average(&self) -> u32 {
        (self.low as u32 + self.high as u32) / 2
    }

    pub fn diff(&self) -> u16 {
        num::abs((self.high as i32) - (self.low as i32)) as u16
    }

    pub fn high(&self) -> u16 {
        self.high
    }

    pub fn low(&self) -> u16 {
        self.low
    }
}

pub trait HighLowReader {
    fn read_high(&self) -> u16;
    fn read_low(&self) -> u16;
}