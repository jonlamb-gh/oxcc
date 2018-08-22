use nucleo_f767zi::hal::can::{BaseID, CanError, DataFrame, ID};

pub const OSCC_STEERING_ENABLE_CAN_ID: u16 = 0x80;
pub const OSCC_STEERING_DISABLE_CAN_ID: u16 = 0x81;
pub const OSCC_STEERING_COMMAND_CAN_ID: u16 = 0x82;
pub const OSCC_STEERING_REPORT_CAN_ID: u16 = 0x83;

pub const OSCC_STEERING_REPORT_CAN_DLC: u8 = 8;

// TODO - enum
pub const OSCC_STEERING_DTC_INVALID_SENSOR_VAL: u8 = 0;
pub const OSCC_STEERING_DTC_OPERATOR_OVERRIDE: u8 = 1;

pub struct OsccSteeringCommand {
    pub torque_request: f32,
}

impl<'a> From<&'a DataFrame> for OsccSteeringCommand {
    fn from(f: &DataFrame) -> Self {
        assert_eq!(u32::from(f.id()), u32::from(OSCC_STEERING_COMMAND_CAN_ID));
        let data = f.data();

        let raw_torque_request: u32 = u32::from(data[2])
            | (u32::from(data[3]) << 8)
            | (u32::from(data[4]) << 16)
            | (u32::from(data[5]) << 24);

        OsccSteeringCommand {
            torque_request: f32::from_bits(raw_torque_request),
        }
    }
}

pub struct OsccSteeringReport {
    pub enabled: bool,
    pub operator_override: bool,
    pub dtcs: u8,
}

pub trait SteeringReportSupplier {
    fn supply_steering_report(&mut self) -> &OsccSteeringReport;
}

pub trait SteeringReportPublisher {
    fn publish_steering_report(
        &mut self,
        brake_report: &OsccSteeringReport,
    ) -> Result<(), CanError>;
}

pub fn default_steering_report_data_frame() -> DataFrame {
    DataFrame::new(ID::BaseID(BaseID::new(OSCC_STEERING_REPORT_CAN_ID)))
}

impl OsccSteeringReport {
    pub fn new() -> Self {
        OsccSteeringReport {
            enabled: false,
            operator_override: false,
            dtcs: 0,
        }
    }
}
