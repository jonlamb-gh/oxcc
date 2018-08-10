use board::ControlCan;
use nucleo_f767zi::hal::can::{BaseID, DataFrame, ID};
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

// TODO - fix this organization
pub struct OsccFaultReportFrame {
    can_frame: DataFrame,
    pub fault_report: OsccFaultReport,
}

impl<'a> From<&'a DataFrame> for OsccFaultReport {
    fn from(f: &DataFrame) -> Self {
        assert_eq!(u32::from(f.id()), OSCC_FAULT_REPORT_CAN_ID as u32);
        let data = f.data();

        let fault_origin_id: u32 = data[2] as u32
            | ((data[3] as u32) << 8)
            | ((data[4] as u32) << 16)
            | ((data[5] as u32) << 24);

        OsccFaultReport {
            fault_origin_id,
            dtcs: data[6],
        }
    }
}

impl OsccFaultReportFrame {
    pub fn new() -> Self {
        OsccFaultReportFrame {
            can_frame: DataFrame::new(ID::BaseID(BaseID::new(OSCC_FAULT_REPORT_CAN_ID))),
            fault_report: OsccFaultReport::new(),
        }
    }

    // TODO - error handling
    pub fn transmit(&mut self, can: &mut ControlCan) {
        self.update_can_frame();

        if let Err(_) = can.transmit(&self.can_frame.into()) {
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
