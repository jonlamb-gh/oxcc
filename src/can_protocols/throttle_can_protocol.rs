use board::ControlCan;
use nucleo_f767zi::can::{BaseID, CanFrame, DataFrame, ID};
use oscc_magic_byte::*;

pub const OSCC_THROTTLE_CAN_ID_INDEX: u16 = 0x90;

pub const OSCC_THROTTLE_ENABLE_CAN_ID: u16 = 0x90;

pub const OSCC_THROTTLE_DISABLE_CAN_ID: u16 = 0x91;

pub const OSCC_THROTTLE_COMMAND_CAN_ID: u16 = 0x92;

pub const OSCC_THROTTLE_REPORT_CAN_ID: u16 = 0x93;

pub const OSCC_THROTTLE_REPORT_CAN_DLC: u8 = 8;

pub struct OsccThrottleEnable {}

pub struct OsccThrottleDisable {}

pub struct OsccThrottleCommand {
    torque_request: f32,
}

pub struct OsccThrottleReport {
    can_frame: DataFrame,
    pub enabled: bool,
    pub operator_override: bool,
    pub dtcs: u8,
}

impl OsccThrottleReport {
    pub fn new() -> Self {
        OsccThrottleReport {
            can_frame: DataFrame::new(ID::BaseID(BaseID::new(OSCC_THROTTLE_REPORT_CAN_ID))),
            enabled: false,
            operator_override: false,
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
            .set_data_length(OSCC_THROTTLE_REPORT_CAN_DLC as _);

        let mut data = self.can_frame.data_as_mut();

        data[0] = OSCC_MAGIC_BYTE_0;
        data[1] = OSCC_MAGIC_BYTE_1;
        data[2] = self.enabled as _;
        data[3] = self.operator_override as _;
        data[4] = self.dtcs;
    }
}
