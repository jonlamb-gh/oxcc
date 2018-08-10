use adc_signal::AdcSignal;
use board::{Board, ADC_SAMPLE_TIME, DAC_SAMPLE_AVERAGE_COUNT};
use num;

pub struct DualSignal {
    high: u16,
    low: u16,
    high_signal: AdcSignal,
    low_signal: AdcSignal,
}

impl DualSignal {
    pub fn new(high: u16, low: u16, high_signal: AdcSignal, low_signal: AdcSignal) -> Self {
        assert_ne!(high_signal, low_signal);
        DualSignal {
            high,
            low,
            high_signal,
            low_signal,
        }
    }

    pub fn update(&mut self, board: &mut Board) {
        self.high = board.analog_read(self.high_signal, ADC_SAMPLE_TIME);
        self.low = board.analog_read(self.low_signal, ADC_SAMPLE_TIME);
    }

    // not sure if the averaging is needed, we might be able to just use a
    // single read with large Cycles480 sample time?
    // https://github.com/jonlamb-gh/oscc/blob/devel/firmware/common/libs/dac/oscc_dac.cpp#L17
    pub fn prevent_signal_discontinuity(&mut self, board: &mut Board) {
        let mut low: u32 = 0;
        let mut high: u32 = 0;

        for _ in 0..DAC_SAMPLE_AVERAGE_COUNT {
            low += board.analog_read(self.low_signal, ADC_SAMPLE_TIME) as u32;
        }

        for _ in 0..DAC_SAMPLE_AVERAGE_COUNT {
            high += board.analog_read(self.high_signal, ADC_SAMPLE_TIME) as u32;
        }

        self.low = (low / DAC_SAMPLE_AVERAGE_COUNT) as _;
        self.high = (high / DAC_SAMPLE_AVERAGE_COUNT) as _;

        board
            .dac
            .set_outputs(self.dac_output_a(), self.dac_output_b());
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

    pub fn dac_output_a(&self) -> u16 {
        self.low
    }

    pub fn dac_output_b(&self) -> u16 {
        self.high
    }
}
