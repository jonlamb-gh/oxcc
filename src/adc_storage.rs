use core::ops;

pub enum Signal {
    AcceleratorPositionSensorHigh,
    AcceleratorPositionSensorLow,
    TorqueSensorHigh,
    TorqueSensorLow,
    BrakePedalPositionSensorHigh,
    BrakePedalPositionSensorLow,
}

pub struct AdcStorage {
    samples: [u16; 6],
    count: u64,
}

impl AdcStorage {
    pub const fn new() -> Self {
        AdcStorage {
            samples: [0; 6],
            count: 0,
        }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn count(&self) -> u64 {
        self.count
    }
}

impl ops::Index<usize> for AdcStorage {
    type Output = u16;

    fn index(&self, i: usize) -> &u16 {
        &self.samples[i]
    }
}

impl ops::Index<Signal> for AdcStorage {
    type Output = u16;

    fn index(&self, s: Signal) -> &u16 {
        &self.samples[s as usize]
    }
}

impl ops::IndexMut<usize> for AdcStorage {
    fn index_mut(&mut self, i: usize) -> &mut u16 {
        &mut self.samples[i]
    }
}

impl ops::IndexMut<Signal> for AdcStorage {
    fn index_mut(&mut self, s: Signal) -> &mut u16 {
        &mut self.samples[s as usize]
    }
}
