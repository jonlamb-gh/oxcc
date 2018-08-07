use board::ControlCan;
use nucleo_f767zi::can::{BaseID, DataFrame, ID};
use oscc_magic_byte::*;

pub const OSCC_STEERING_ENABLE_CAN_ID: u16 = 0x80;
pub const OSCC_STEERING_DISABLE_CAN_ID: u16 = 0x81;
pub const OSCC_STEERING_COMMAND_CAN_ID: u16 = 0x82;
pub const OSCC_STEERING_REPORT_CAN_ID: u16 = 0x83;

pub const OSCC_STEERING_REPORT_CAN_DLC: u8 = 8;

// TODO - enum
pub const OSCC_STEERING_DTC_INVALID_SENSOR_VAL: u8 = 0;
pub const OSCC_STEERING_DTC_OPERATOR_OVERRIDE: u8 = 1;
