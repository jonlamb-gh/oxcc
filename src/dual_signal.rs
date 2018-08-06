pub struct DualSignal {
    high: u16,
    low: u16,
}

impl DualSignal {
    pub const fn new(high: u16, low: u16) -> Self {
        DualSignal { high, low }
    }

    pub fn update(&mut self, high: u16, low: u16) {
        self.high = high;
        self.low = low;
    }

    pub fn average(&self) -> u32 {
        (self.low as u32 + self.high as u32) / 2
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
