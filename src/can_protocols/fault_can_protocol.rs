use board::ControlCan;
use nucleo_f767zi::can::{BaseID, CanFrame, DataFrame, ID};
use oscc_magic_byte::*;

pub const OSCC_FAULT_CAN_ID_INDEX: u16 = 0xA0;

pub const OSCC_FAULT_REPORT_CAN_ID: u16 = 0xAF;

pub const OSCC_FAULT_REPORT_CAN_DLC: u8 = 8;

// TODO - enum
pub const FAULT_ORIGIN_BRAKE: u32 = 0;
pub const FAULT_ORIGIN_STEERING: u32 = 1;
pub const FAULT_ORIGIN_THROTTLE: u32 = 2;

pub struct OsccFaultReport {
    can_frame: DataFrame,
    pub fault_origin_id: u32,
    pub dtcs: u8,
}

impl OsccFaultReport {
    pub fn new() -> Self {
        OsccFaultReport {
            can_frame: DataFrame::new(ID::BaseID(BaseID::new(OSCC_FAULT_REPORT_CAN_ID))),
            fault_origin_id: FAULT_ORIGIN_BRAKE,
            dtcs: 0,
        }
    }

    // TODO - error handling
    pub fn transmit(&mut self, can: &mut ControlCan) {
        self.update_can_frame();
        can.transmit(&self.can_frame.into()).unwrap();
    }

    fn update_can_frame(&mut self) {
        self.can_frame
            .set_data_length(OSCC_FAULT_REPORT_CAN_DLC as _);

        let mut data = self.can_frame.data_as_mut();

        data[0] = OSCC_MAGIC_BYTE_0;
        data[1] = OSCC_MAGIC_BYTE_1;
        data[2] = (self.fault_origin_id & 0xFF) as _;
        data[3] = ((self.fault_origin_id >> 8) & 0xFF) as _;
        data[4] = ((self.fault_origin_id >> 16) & 0xFF) as _;
        data[5] = ((self.fault_origin_id >> 24) & 0xFF) as _;
        data[6] = self.dtcs;
    }
}
