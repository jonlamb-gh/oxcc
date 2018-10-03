//! Throttle CAN protocol

use nucleo_f767zi::hal::can::{BaseID, CanError, DataFrame, ID};

pub const OSCC_THROTTLE_ENABLE_CAN_ID: u16 = 0x90;
pub const OSCC_THROTTLE_DISABLE_CAN_ID: u16 = 0x91;
pub const OSCC_THROTTLE_COMMAND_CAN_ID: u16 = 0x92;
pub const OSCC_THROTTLE_REPORT_CAN_ID: u16 = 0x93;

pub const OSCC_THROTTLE_REPORT_CAN_DLC: u8 = 8;

// TODO - enum
pub const OSCC_THROTTLE_DTC_INVALID_SENSOR_VAL: u8 = 0;
pub const OSCC_THROTTLE_DTC_OPERATOR_OVERRIDE: u8 = 1;

pub struct OsccThrottleCommand {
    pub torque_request: f32,
}

impl<'a> From<&'a DataFrame> for OsccThrottleCommand {
    fn from(f: &DataFrame) -> Self {
        assert_eq!(u32::from(f.id()), u32::from(OSCC_THROTTLE_COMMAND_CAN_ID));
        let data = f.data();

        let raw_torque_request: u32 = u32::from(data[2])
            | (u32::from(data[3]) << 8)
            | (u32::from(data[4]) << 16)
            | (u32::from(data[5]) << 24);

        OsccThrottleCommand {
            torque_request: f32::from_bits(raw_torque_request),
        }
    }
}

pub struct OsccThrottleReport {
    pub enabled: bool,
    pub operator_override: bool,
    pub dtcs: u8,
}

pub trait ThrottleReportSupplier {
    fn supply_throttle_report(&mut self) -> &OsccThrottleReport;
}

pub trait ThrottleReportPublisher {
    fn publish_throttle_report(
        &mut self,
        throttle_report: &OsccThrottleReport,
    ) -> Result<(), CanError>;
}

pub fn default_throttle_report_data_frame() -> DataFrame {
    DataFrame::new(ID::BaseID(BaseID::new(OSCC_THROTTLE_REPORT_CAN_ID)))
}

impl OsccThrottleReport {
    pub fn new() -> Self {
        OsccThrottleReport {
            enabled: false,
            operator_override: false,
            dtcs: 0,
        }
    }
}
