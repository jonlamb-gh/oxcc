//! Brake CAN protocol

use nucleo_f767zi::hal::can::{BaseID, CanError, DataFrame, ID};

pub const OSCC_BRAKE_ENABLE_CAN_ID: u16 = 0x70;
pub const OSCC_BRAKE_DISABLE_CAN_ID: u16 = 0x71;
pub const OSCC_BRAKE_COMMAND_CAN_ID: u16 = 0x72;
pub const OSCC_BRAKE_REPORT_CAN_ID: u16 = 0x73;

pub const OSCC_BRAKE_REPORT_CAN_DLC: u8 = 8;

// TODO - enum
pub const OSCC_BRAKE_DTC_INVALID_SENSOR_VAL: u8 = 0;
pub const OSCC_BRAKE_DTC_OPERATOR_OVERRIDE: u8 = 1;

pub struct OsccBrakeCommand {
    pub pedal_command: f32,
}

impl<'a> From<&'a DataFrame> for OsccBrakeCommand {
    fn from(f: &DataFrame) -> Self {
        assert_eq!(u32::from(f.id()), u32::from(OSCC_BRAKE_COMMAND_CAN_ID));
        let data = f.data();

        let raw_brake_request: u32 = u32::from(data[2])
            | (u32::from(data[3]) << 8)
            | (u32::from(data[4]) << 16)
            | (u32::from(data[5]) << 24);

        OsccBrakeCommand {
            pedal_command: f32::from_bits(raw_brake_request),
        }
    }
}

pub struct OsccBrakeReport {
    pub enabled: bool,
    pub operator_override: bool,
    pub dtcs: u8,
}

pub trait BrakeReportSupplier {
    fn supply_brake_report(&mut self) -> &OsccBrakeReport;
}

pub trait BrakeReportPublisher {
    fn publish_brake_report(&mut self, brake_report: &OsccBrakeReport) -> Result<(), CanError>;
}

pub fn default_brake_report_data_frame() -> DataFrame {
    DataFrame::new(ID::BaseID(BaseID::new(OSCC_BRAKE_REPORT_CAN_ID)))
}

impl OsccBrakeReport {
    pub fn new() -> Self {
        OsccBrakeReport {
            enabled: false,
            operator_override: false,
            dtcs: 0,
        }
    }
}
