//! Kia Niro vehicle configuration data
//!
//! **WARNING**
//!
//! The values listed here are carefully tested to ensure that the vehicle's
//! components are not actuated outside of the range of what they can handle.
//! By changing any of these values you risk attempting to actuate outside of
//! the vehicle's valid range. This can cause damage to the hardware and/or a
//! vehicle fault. Clearing this fault state requires additional tools.
//!
//! It is NOT recommended to modify any of these values without expert
//! knowledge.

#![allow(dead_code)]

use ranges;
use typenum::consts::*;
type U1723 = op!{U1000 + U723};
type U3358 = op!{U1000 + U1000 + U1000 + U358};
type U3440 = op!{U1000 + U1000 + U1000 + U440};
type U3446 = op!{U1000 + U1000 + U1000 + U446};

// ****************************************************************************
// OBD MESSAGES
// ****************************************************************************

/// ID of the Kia Niro's OBD steering wheel angle CAN frame.
pub const KIA_SOUL_OBD_STEERING_WHEEL_ANGLE_CAN_ID: u16 = 0x2B0;

/// ID of the Kia Niro's OBD wheel speed CAN frame.
pub const KIA_SOUL_OBD_WHEEL_SPEED_CAN_ID: u16 = 0x386;

/// ID of the Kia Niro's OBD brake pressure CAN frame.
pub const KIA_SOUL_OBD_BRAKE_PRESSURE_CAN_ID: u16 = 0x220;

/// ID of the Kia Niro's OBD speed CAN frame.
pub const KIA_SOUL_OBD_SPEED_CAN_ID: u16 = 0x371;

/// Factor to scale OBD steering angle to degrees
pub const KIA_SOUL_OBD_STEERING_ANGLE_SCALAR: f32 = 0.1;

// ****************************************************************************
// VEHICLE AND BOARD PARAMETERS
// ****************************************************************************

/// Number of steps per volt corresponding to 4096 steps (2^12) across
/// 5 volts.
pub const STEPS_PER_VOLT: f32 = 819.2;

/// Length of time in ms for delay of signal reads to ensure fault is
/// outside the range of noise in the signal.
pub const FAULT_HYSTERESIS: u32 = 150;

// ****************************************************************************
// BRAKE MODULE
// ****************************************************************************

/// Minimum allowable brake value.
pub const MINIMUM_BRAKE_COMMAND: f32 = 0.0;

/// Maximum allowable brake value.
pub const MAXIMUM_BRAKE_COMMAND: f32 = 1.0;

/// Minimum allowed voltage for the high spoof signal voltage. \[volts\]
pub const BRAKE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN: f32 = 0.609;

/// Maximum allowed voltage for the high spoof signal voltage. \[volts\]
pub const BRAKE_SPOOF_LOW_SIGNAL_VOLTAGE_MAX: f32 = 2.880;

/// Minimum allowed voltage for the low spoof signal voltage. \[volts\]
pub const BRAKE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN: f32 = 0.279;

/// Maximum allowed voltage for the low spoof signal voltage. \[volts\]
pub const BRAKE_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX: f32 = 1.386;

/// Minimum allowed value for the high spoof signal value. \[steps\]
/// Equal to BRAKE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN * STEPS_PER_VOLT.
pub const BRAKE_SPOOF_LOW_SIGNAL_RANGE_MIN: u16 = 499;

/// Minimum allowed value for the high spoof signal value. \[steps\]
/// Equal to BRAKE_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX * STEPS_PER_VOLT.
pub const BRAKE_SPOOF_LOW_SIGNAL_RANGE_MAX: u16 = 2359;

/// Minimum allowed value for the low spoof signal value. \[steps\]
/// Equal to BRAKE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN * STEPS_PER_VOLT.
pub const BRAKE_SPOOF_HIGH_SIGNAL_RANGE_MIN: u16 = 229;

/// Minimum allowed value for the low spoof signal value. \[steps\]
/// Equal to BRAKE_SPOOF_LOW_SIGNAL_VOLTAGE_MAX * STEPS_PER_VOLT.
pub const BRAKE_SPOOF_HIGH_SIGNAL_RANGE_MAX: u16 = 1135;

/// Calculation to convert a brake position to a low spoof voltage.
pub const fn brake_position_to_volts_low(position: f32) -> f32 {
    position * (BRAKE_SPOOF_LOW_SIGNAL_VOLTAGE_MAX - BRAKE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN)
        + BRAKE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN
}

/// Calculation to convert a brake position to a high spoof voltage.
pub const fn brake_position_to_volts_high(position: f32) -> f32 {
    position * (BRAKE_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX - BRAKE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN)
        + BRAKE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN
}

/// Value of the accelerator position that indicates operator
/// override. \[steps\]
pub const BRAKE_PEDAL_OVERRIDE_THRESHOLD: u16 = 200 << 2;

/// Minimum value of the high spoof signal that activates the brake
/// lights. \[steps\]
pub const BRAKE_LIGHT_SPOOF_HIGH_THRESHOLD: u16 = 300;

/// Minimum value of the low spoof signal that activates the brake
/// lights. \[steps\]
pub const BRAKE_LIGHT_SPOOF_LOW_THRESHOLD: u16 = 600;

// ****************************************************************************
// STEERING MODULE
// ****************************************************************************

/// Minimum allowable torque value.
pub const MINIMUM_TORQUE_COMMAND: f32 = -12.8;

/// Maximum allowable torque value.
pub const MAXIMUM_TORQUE_COMMAND: f32 = 12.7;

/// Minimum allowable steering DAC output. \[volts\]
pub const STEERING_SPOOF_LOW_SIGNAL_VOLTAGE_MIN: f32 = 0.80;

/// Maximum allowable steering DAC output. \[volts\]
pub const STEERING_SPOOF_LOW_SIGNAL_VOLTAGE_MAX: f32 = 4.10;

/// Minimum allowable steering DAC output. \[volts\]
pub const STEERING_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN: f32 = 0.90;

/// Maximum allowable steering DAC output. \[volts\]
pub const STEERING_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX: f32 = 4.20;

/// Minimum allowed value for the low spoof signal value. \[steps\]
/// Equal to STEERING_SPOOF_LOW_SIGNAL_VOLTAGE_MIN * STEPS_PER_VOLT.
pub const STEERING_SPOOF_LOW_SIGNAL_RANGE_MIN: u16 = 656;

/// Minimum allowed value for the low spoof signal value. \[steps\]
/// Equal to STEERING_SPOOF_LOW_SIGNAL_VOLTAGE_MAX * STEPS_PER_VOLT.
pub const STEERING_SPOOF_LOW_SIGNAL_RANGE_MAX: u16 = 3358;

pub type SteeringSpoofLowSignal = ranges::Bounded<u16, U656, U3358>;

/// Minimum allowed value for the low spoof signal value. \[steps\]
/// Equal to STEERING_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN * STEPS_PER_VOLT.
pub const STEERING_SPOOF_HIGH_SIGNAL_RANGE_MIN: u16 = 738;

/// Minimum allowed value for the low spoof signal value. \[steps\]
/// Equal to STEERING_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX * STEPS_PER_VOLT.
pub const STEERING_SPOOF_HIGH_SIGNAL_RANGE_MAX: u16 = 3440;

pub type SteeringSpoofHighSignal = ranges::Bounded<u16, U738, U3440>;

/// Scalar value for the low spoof signal taken from a calibration
/// curve.
pub const TORQUE_SPOOF_LOW_SIGNAL_CALIBRATION_CURVE_SCALE: f32 = 0.135;

/// Offset value for the low spoof signal taken from a calibration
/// curve.
pub const TORQUE_SPOOF_LOW_SIGNAL_CALIBRATION_CURVE_OFFSET: f32 = 2.39;

/// Scalar value for the high spoof signal taken from a calibration
/// curve.
pub const TORQUE_SPOOF_HIGH_SIGNAL_CALIBRATION_CURVE_SCALE: f32 = -0.145;

/// Offset value for the high spoof signal taken from a calibration
/// curve.
pub const TORQUE_SPOOF_HIGH_SIGNAL_CALIBRATION_CURVE_OFFSET: f32 = 2.42;

/// Minimum allowed value for the high spoof signal value.
pub const fn steering_torque_to_volts_low(torque: f32) -> f32 {
    (TORQUE_SPOOF_HIGH_SIGNAL_CALIBRATION_CURVE_SCALE * torque)
        + TORQUE_SPOOF_HIGH_SIGNAL_CALIBRATION_CURVE_OFFSET
}

/// Calculation to convert a steering torque to a low spoof value.
pub const fn steering_torque_to_volts_high(torque: f32) -> f32 {
    (TORQUE_SPOOF_LOW_SIGNAL_CALIBRATION_CURVE_SCALE * torque)
        + TORQUE_SPOOF_LOW_SIGNAL_CALIBRATION_CURVE_OFFSET
}

/// Value of torque sensor difference that indicates likely operator
///        override.
pub const TORQUE_DIFFERENCE_OVERRIDE_THRESHOLD: u16 = 1600;

// ****************************************************************************
// THROTTLE MODULE
// ****************************************************************************

/// Minimum allowable throttle value.
pub const MINIMUM_THROTTLE_COMMAND: f32 = 0.0;

/// Maximum allowable throttle value.
pub const MAXIMUM_THROTTLE_COMMAND: f32 = 1.0;

/// Minimum allowed voltage for the low spoof signal voltage. \[volts\]
pub const THROTTLE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN: f32 = 0.380;

/// Maximum allowed voltage for the low spoof signal voltage. \[volts\]
pub const THROTTLE_SPOOF_LOW_SIGNAL_VOLTAGE_MAX: f32 = 2.104;

/// Minimum allowed voltage for the high spoof signal voltage. \[volts\]
pub const THROTTLE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN: f32 = 0.757;

/// Maximum allowed voltage for the high spoof signal voltage. \[volts\]
pub const THROTTLE_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX: f32 = 4.207;

/// Minimum allowed value for the low spoof signal value. \[steps\]
/// Equal to THROTTLE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN * STEPS_PER_VOLT.
pub const THROTTLE_SPOOF_LOW_SIGNAL_RANGE_MIN: u16 = 311;

/// Minimum allowed value for the low spoof signal value. \[steps\]
/// Equal to THROTTLE_SPOOF_LOW_SIGNAL_VOLTAGE_MAX * STEPS_PER_VOLT.
pub const THROTTLE_SPOOF_LOW_SIGNAL_RANGE_MAX: u16 = 1723;

pub type ThrottleSpoofLowSignal = ranges::Bounded<u16, U311, U1723>;

/// Minimum allowed value for the low spoof signal value. \[steps\]
/// Equal to THROTTLE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN * STEPS_PER_VOLT.
pub const THROTTLE_SPOOF_HIGH_SIGNAL_RANGE_MIN: u16 = 620;

/// Minimum allowed value for the low spoof signal value. \[steps\]
/// Equal to THROTTLE_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX * STEPS_PER_VOLT.
pub const THROTTLE_SPOOF_HIGH_SIGNAL_RANGE_MAX: u16 = 3446;

pub type ThrottleSpoofHighSignal = ranges::Bounded<u16, U620, U3446>;

/// Calculation to convert a throttle position to a low spoof voltage.
pub const fn throttle_position_to_volts_low(position: f32) -> f32 {
    position * (THROTTLE_SPOOF_LOW_SIGNAL_VOLTAGE_MAX - THROTTLE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN)
        + THROTTLE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN
}

/// Calculation to convert a throttle position to a high spoof voltage.
pub const fn throttle_position_to_volts_high(position: f32) -> f32 {
    position * (THROTTLE_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX - THROTTLE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN)
        + THROTTLE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN
}

/// Value of the accelerator position that indicates operator
/// override. \[steps\]
pub const ACCELERATOR_OVERRIDE_THRESHOLD: u32 = 185 << 2;
