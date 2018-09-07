use dac_mcp4922::Mcp4922;
use nucleo_f767zi::hal::can::Can;
use nucleo_f767zi::hal::gpio::gpioa::{PA15, PA4, PA5, PA6, PA7};
use nucleo_f767zi::hal::gpio::gpiob::{PB10, PB12, PB13, PB15, PB4};
use nucleo_f767zi::hal::gpio::gpioc::{PC10, PC11, PC12, PC2};
use nucleo_f767zi::hal::gpio::gpiod::{PD0, PD1, PD10, PD11, PD12, PD13};
use nucleo_f767zi::hal::gpio::{Output, PushPull, AF5, AF9};
use nucleo_f767zi::hal::spi::Spi;
use nucleo_f767zi::hal::stm32f7x7::{
    CAN1, CAN2, SPI1, SPI2, SPI3, TIM2, TIM3, TIM4, TIM5, TIM6, TIM7,
};
use nucleo_f767zi::hal::timer::Timer;
use nucleo_f767zi::{
    AnalogInput0Pin, AnalogInput1Pin, AnalogInput2Pin, AnalogInput4Pin, AnalogInput5Pin,
    AnalogInput6Pin,
};

pub type CanPublishTimer = Timer<TIM2>;
pub type BrakeGroundedFaultTimer = Timer<TIM3>;
pub type BrakeOverrideFaultTimer = Timer<TIM4>;
pub type ThrottleGroundedFaultTimer = Timer<TIM5>;
pub type ThrottleOverrideFaultTimer = Timer<TIM6>;
pub type SteeringGroundedFaultTimer = Timer<TIM7>;

pub type ControlCan = Can<CAN1, (PD1<AF9>, PD0<AF9>)>;
pub type ObdCan = Can<CAN2, (PB13<AF9>, PB12<AF9>)>;

pub type BrakeSpi = Spi<SPI1, (PA5<AF5>, PA6<AF5>, PA7<AF5>)>;
pub type ThrottleSpi = Spi<SPI2, (PB10<AF5>, PC2<AF5>, PB15<AF5>)>;
pub type SteeringSpi = Spi<SPI3, (PC10<AF5>, PC11<AF5>, PC12<AF5>)>;

pub type BrakeSpoofEnablePin = PD12<Output<PushPull>>;
pub type BrakeLightEnablePin = PD13<Output<PushPull>>;
// AIN pins chosen to allow brake module to own ADC1
pub type BrakePedalPositionSensorHighPin = AnalogInput0Pin;
pub type BrakePedalPositionSensorLowPin = AnalogInput1Pin;
pub type BrakeSpiSckPin = PA5<AF5>;
pub type BrakeSpiMisoPin = PA6<AF5>;
pub type BrakeSpiMosiPin = PA7<AF5>;
pub type BrakeSpiNssPin = PA4<Output<PushPull>>;

pub type BrakeDac = Mcp4922<BrakeSpi, BrakeSpiNssPin>;

pub type ThrottleSpoofEnablePin = PD10<Output<PushPull>>;
// AIN pins chosen to allow throttle module to own ADC2
pub type AcceleratorPositionSensorHighPin = AnalogInput2Pin;
pub type AcceleratorPositionSensorLowPin = AnalogInput6Pin;
pub type ThrottleSpiSckPin = PB10<AF5>;
pub type ThrottleSpiMisoPin = PC2<AF5>;
pub type ThrottleSpiMosiPin = PB15<AF5>;
pub type ThrottleSpiNssPin = PB4<Output<PushPull>>;

pub type ThrottleDac = Mcp4922<ThrottleSpi, ThrottleSpiNssPin>;

pub type SteeringSpoofEnablePin = PD11<Output<PushPull>>;
// AIN pins chosen to allow steering module to own ADC3
pub type TorqueSensorHighPin = AnalogInput4Pin;
pub type TorqueSensorLowPin = AnalogInput5Pin;
pub type SteeringSpiSckPin = PC10<AF5>;
pub type SteeringSpiMisoPin = PC11<AF5>;
pub type SteeringSpiMosiPin = PC12<AF5>;
pub type SteeringSpiNssPin = PA15<Output<PushPull>>;

pub type SteeringDac = Mcp4922<SteeringSpi, SteeringSpiNssPin>;

pub struct BrakePins {
    pub spoof_enable: BrakeSpoofEnablePin,
    pub brake_light_enable: BrakeLightEnablePin,
    pub pedal_pos_sensor_high: BrakePedalPositionSensorHighPin,
    pub pedal_pos_sensor_low: BrakePedalPositionSensorLowPin,
}

pub struct ThrottlePins {
    pub spoof_enable: ThrottleSpoofEnablePin,
    pub accel_pos_sensor_high: AcceleratorPositionSensorHighPin,
    pub accel_pos_sensor_low: AcceleratorPositionSensorLowPin,
}

pub struct SteeringPins {
    pub spoof_enable: SteeringSpoofEnablePin,
    pub torque_sensor_high: TorqueSensorHighPin,
    pub torque_sensor_low: TorqueSensorLowPin,
}
