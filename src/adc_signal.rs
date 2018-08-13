use nucleo_f767zi::hal::adc::AdcChannel;

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
