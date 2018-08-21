use board::ControlCan;
use nucleo_f767zi::hal::can::{BaseID, DataFrame, ID};
use oscc_magic_byte::*;

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
            pedal_command: raw_brake_request as f32,
        }
    }
}

pub struct OsccBrakeReport {
    can_frame: DataFrame,
    pub enabled: bool,
    pub operator_override: bool,
    pub dtcs: u8,
}

impl OsccBrakeReport {
    pub fn new() -> Self {
        OsccBrakeReport {
            can_frame: DataFrame::new(ID::BaseID(BaseID::new(OSCC_BRAKE_REPORT_CAN_ID))),
            enabled: false,
            operator_override: false,
            dtcs: 0,
        }
    }

    // TODO - error handling
    pub fn transmit(&mut self, can: &mut ControlCan) {
        self.update_can_frame();

        if can.transmit(&self.can_frame.into()).is_err() {
            // TODO
        }
    }

    fn update_can_frame(&mut self) {
        self.can_frame
            .set_data_length(OSCC_BRAKE_REPORT_CAN_DLC as _);

        let data = self.can_frame.data_as_mut();

        data[0] = OSCC_MAGIC_BYTE_0;
        data[1] = OSCC_MAGIC_BYTE_1;
        data[2] = self.enabled as _;
        data[3] = self.operator_override as _;
        data[4] = self.dtcs;
    }
}
