use nucleo_f767zi::hal::can::Can;
use nucleo_f767zi::hal::gpio::gpiob::{PB12, PB13};
use nucleo_f767zi::hal::gpio::gpiod::{PD0, PD1, PD10, PD11, PD12, PD13};
use nucleo_f767zi::hal::gpio::{AF9, Output, PushPull};
use nucleo_f767zi::hal::stm32f7x7::{CAN1, CAN2, TIM2};
use nucleo_f767zi::hal::timer::Timer;
use nucleo_f767zi::{
    AnalogInput0Pin, AnalogInput1Pin, AnalogInput2Pin, AnalogInput3Pin, AnalogInput4Pin,
    AnalogInput5Pin,
};

pub type CanPublishTimer = Timer<TIM2>;

pub type ControlCan = Can<CAN1, (PD1<AF9>, PD0<AF9>)>;
pub type ObdCan = Can<CAN2, (PB13<AF9>, PB12<AF9>)>;

pub type BrakeSpoofEnablePin = PD12<Output<PushPull>>;
pub type BrakeLightEnablePin = PD13<Output<PushPull>>;
pub type BrakePedalPositionSensorHighPin = AnalogInput4Pin;
pub type BrakePedalPositionSensorLowPin = AnalogInput5Pin;

pub type ThrottleSpoofEnablePin = PD10<Output<PushPull>>;
pub type AcceleratorPositionSensorHighPin = AnalogInput0Pin;
pub type AcceleratorPositionSensorLowPin = AnalogInput1Pin;

pub type SteeringSpoofEnablePin = PD11<Output<PushPull>>;
pub type TorqueSensorHighPin = AnalogInput2Pin;
pub type TorqueSensorLowPin = AnalogInput3Pin;

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
