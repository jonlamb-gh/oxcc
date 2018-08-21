use board::ControlCan;
use nucleo_f767zi::hal::can::{BaseID, CanError, DataFrame, ID};
use oscc_magic_byte::*;

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

impl OsccFaultReport {
    pub fn new() -> Self {
        OsccFaultReport {
            fault_origin_id: FAULT_ORIGIN_BRAKE,
            dtcs: 0,
        }
    }
}

pub trait FaultReportSupplier {
    fn supply_fault_report(&mut self) -> &OsccFaultReport;
}

pub trait FaultReportPublisher {
    fn publish_fault_report(&mut self, fault_report: &OsccFaultReport) -> Result<(), CanError>;
}

// TODO - fix this organization
pub struct OsccFaultReportFrame {
    can_frame: DataFrame,
    pub fault_report: OsccFaultReport,
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

pub fn default_fault_report_data_frame() -> DataFrame {
    DataFrame::new(ID::BaseID(BaseID::new(OSCC_FAULT_REPORT_CAN_ID)))
}

impl OsccFaultReportFrame {
    pub fn new() -> Self {
        OsccFaultReportFrame {
            can_frame: default_fault_report_data_frame(),
            fault_report: OsccFaultReport::new(),
        }
    }

    // TODO - error handling
    // TODO - replace with the publisher pattern more completely
    pub fn transmit(&mut self, can: &mut ControlCan) {
        self.update_can_frame();

        if can.transmit(&self.can_frame.into()).is_err() {
            // TODO
        }
    }

    fn update_can_frame(&mut self) {
        self.can_frame
            .set_data_length(OSCC_FAULT_REPORT_CAN_DLC as _);

        let data = self.can_frame.data_as_mut();

        data[0] = OSCC_MAGIC_BYTE_0;
        data[1] = OSCC_MAGIC_BYTE_1;
        data[2] = (self.fault_report.fault_origin_id & 0xFF) as _;
        data[3] = ((self.fault_report.fault_origin_id >> 8) & 0xFF) as _;
        data[4] = ((self.fault_report.fault_origin_id >> 16) & 0xFF) as _;
        data[5] = ((self.fault_report.fault_origin_id >> 24) & 0xFF) as _;
        data[6] = self.fault_report.dtcs;
    }
}
