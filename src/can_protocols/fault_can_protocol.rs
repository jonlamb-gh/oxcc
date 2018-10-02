//! Fault CAN protocol

use nucleo_f767zi::hal::can::{BaseID, CanError, DataFrame, ID};

pub const OSCC_FAULT_REPORT_CAN_ID: u16 = 0xAF;

pub const OSCC_FAULT_REPORT_CAN_DLC: u8 = 8;

// TODO - enum
pub const FAULT_ORIGIN_BRAKE: u32 = 0;
pub const FAULT_ORIGIN_STEERING: u32 = 1;
pub const FAULT_ORIGIN_THROTTLE: u32 = 2;

// TODO - fix this organization
pub struct OsccFaultReport {
    pub fault_origin_id: u32,
    pub dtcs: u8,
}

impl<'a> From<&'a DataFrame> for OsccFaultReport {
    fn from(f: &DataFrame) -> Self {
        assert_eq!(u32::from(f.id()), u32::from(OSCC_FAULT_REPORT_CAN_ID));
        let data = f.data();

        let fault_origin_id: u32 = u32::from(data[2])
            | (u32::from(data[3]) << 8)
            | (u32::from(data[4]) << 16)
            | (u32::from(data[5]) << 24);

        OsccFaultReport {
            fault_origin_id,
            dtcs: data[6],
        }
    }
}

pub trait FaultReportSupplier {
    fn supply_fault_report(&mut self) -> &OsccFaultReport;
}

pub trait FaultReportPublisher {
    fn publish_fault_report(&mut self, fault_report: &OsccFaultReport) -> Result<(), CanError>;
}

pub fn default_fault_report_data_frame() -> DataFrame {
    DataFrame::new(ID::BaseID(BaseID::new(OSCC_FAULT_REPORT_CAN_ID)))
}
