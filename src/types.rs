use nucleo_f767zi::hal::can::Can;
use nucleo_f767zi::hal::gpio::gpioa::PA3;
use nucleo_f767zi::hal::gpio::gpiob::{PB12, PB13};
use nucleo_f767zi::hal::gpio::gpioc::{PC0, PC3};
use nucleo_f767zi::hal::gpio::gpiod::{PD0, PD1, PD10, PD11, PD12, PD13};
use nucleo_f767zi::hal::gpio::gpiof::{PF10, PF3, PF5};
use nucleo_f767zi::hal::gpio::{AF7, Analog, Input, Output, PushPull};
use nucleo_f767zi::hal::stm32f7x7::{CAN1, CAN2, TIM2};
use nucleo_f767zi::hal::timer::Timer;

pub type CanPublishTimer = Timer<TIM2>;

pub type ControlCan = Can<CAN1, (PD1<AF7>, PD0<AF7>)>;
pub type ObdCan = Can<CAN2, (PB13<AF7>, PB12<AF7>)>;

pub type BrakeSpoofEnablePin = PD12<Output<PushPull>>;
pub type BrakeLightEnablePin = PD13<Output<PushPull>>;
pub type BrakePedalPositionSensorHighPin = PF5<Input<Analog>>; // ADC3_IN15
pub type BrakePedalPositionSensorLowPin = PF10<Input<Analog>>; // ADC3_IN8

pub type ThrottleSpoofEnablePin = PD10<Output<PushPull>>;
pub type AcceleratorPositionSensorHighPin = PA3<Input<Analog>>; // ADC123_IN3
pub type AcceleratorPositionSensorLowPin = PC0<Input<Analog>>; // ADC123_IN10

pub type SteeringSpoofEnablePin = PD11<Output<PushPull>>;
pub type TorqueSensorHighPin = PC3<Input<Analog>>; // ADC123_IN13
pub type TorqueSensorLowPin = PF3<Input<Analog>>; // ADC3_IN9

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
