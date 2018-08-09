#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AdcSampleTime {
    Cycles3,
    Cycles15,
    Cycles28,
    Cycles56,
    Cycles84,
    Cycles112,
    Cycles144,
    Cycles480,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AdcChannel {
    Adc123In3,
    Adc123In10,
    Adc123In13,
    Adc3In9,
    Adc3In15,
    Adc3In8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AdcSignal {
    AcceleratorPositionSensorHigh,
    AcceleratorPositionSensorLow,
    TorqueSensorHigh,
    TorqueSensorLow,
    BrakePedalPositionSensorHigh,
    BrakePedalPositionSensorLow,
}

impl From<AdcSignal> for AdcChannel {
    fn from(s: AdcSignal) -> Self {
        match s {
            AdcSignal::AcceleratorPositionSensorHigh => AdcChannel::Adc123In3,
            AdcSignal::AcceleratorPositionSensorLow => AdcChannel::Adc123In10,
            AdcSignal::TorqueSensorHigh => AdcChannel::Adc123In13,
            AdcSignal::TorqueSensorLow => AdcChannel::Adc3In9,
            AdcSignal::BrakePedalPositionSensorHigh => AdcChannel::Adc3In15,
            AdcSignal::BrakePedalPositionSensorLow => AdcChannel::Adc3In8,
        }
    }
}

impl From<AdcSampleTime> for u8 {
    fn from(s: AdcSampleTime) -> u8 {
        match s {
            AdcSampleTime::Cycles3 => 0b000,
            AdcSampleTime::Cycles15 => 0b001,
            AdcSampleTime::Cycles28 => 0b010,
            AdcSampleTime::Cycles56 => 0b011,
            AdcSampleTime::Cycles84 => 0b100,
            AdcSampleTime::Cycles112 => 0b101,
            AdcSampleTime::Cycles144 => 0b110,
            AdcSampleTime::Cycles480 => 0b111,
        }
    }
}

impl From<AdcChannel> for u8 {
    fn from(c: AdcChannel) -> u8 {
        match c {
            AdcChannel::Adc123In3 => 3,
            AdcChannel::Adc123In10 => 10,
            AdcChannel::Adc123In13 => 13,
            AdcChannel::Adc3In9 => 9,
            AdcChannel::Adc3In15 => 15,
            AdcChannel::Adc3In8 => 8,
        }
    }
}
