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
            // throttle module owns ADC2
            AdcSignal::AcceleratorPositionSensorHigh => AdcChannel::Adc123In13,
            AdcSignal::AcceleratorPositionSensorLow => AdcChannel::Adc12In9,
            // steering module owns ADC3
            AdcSignal::TorqueSensorHigh => AdcChannel::Adc3In15,
            AdcSignal::TorqueSensorLow => AdcChannel::Adc3In8,
            // brake module owns ADC1
            AdcSignal::BrakePedalPositionSensorHigh => AdcChannel::Adc123In3,
            AdcSignal::BrakePedalPositionSensorLow => AdcChannel::Adc123In10,
        }
    }
}
