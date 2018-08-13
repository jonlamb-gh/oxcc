use brake_can_protocol::*;
use fault_can_protocol::*;
use nucleo_f767zi::hal::can::{CanFilterConfig, FilterMode, FilterScale, RxFifo};
use steering_can_protocol::*;
use throttle_can_protocol::*;

// TODO feature gate vehicles
use kial_soul_ev::*;

// TODO - docs on priority ordering in ID list mode
// can we make a pub type instead?
// CanFilterConfig { enabled: true, ..Default::default() }

pub fn gather_control_can_filters() -> [CanFilterConfig; 3] {
    // filter 0 is the highest priority filter in ID list mode
    // it stores the disable control IDs for throttle, brake, steering
    // and the fault report ID
    // FIFO_0
    let mut f0 = CanFilterConfig::default();
    f0.filter_number = 0;
    f0.enabled = true;
    f0.mode = FilterMode::IdList;
    f0.fifo_assignment = RxFifo::Fifo0;
    f0.scale = FilterScale::Fs16Bit;
    f0.filter_mask_id_low = (OSCC_THROTTLE_DISABLE_CAN_ID << 5) as _;
    f0.filter_id_low = (OSCC_BRAKE_DISABLE_CAN_ID << 5) as _;
    f0.filter_mask_id_high = (OSCC_STEERING_DISABLE_CAN_ID << 5) as _;
    f0.filter_id_high = (OSCC_FAULT_REPORT_CAN_ID << 5) as _;

    // filter 1 stores the control command IDs for brake, throttle, and steering
    // FIFO_1
    let mut f1 = CanFilterConfig::default();
    f1.filter_number = 1;
    f1.enabled = true;
    f1.mode = FilterMode::IdList;
    f1.fifo_assignment = RxFifo::Fifo1;
    f1.scale = FilterScale::Fs16Bit;
    f1.filter_mask_id_low = (OSCC_BRAKE_COMMAND_CAN_ID << 5) as _;
    f1.filter_id_low = (OSCC_THROTTLE_COMMAND_CAN_ID << 5) as _;
    f1.filter_mask_id_high = (OSCC_STEERING_COMMAND_CAN_ID << 5) as _;
    f1.filter_id_high = 0;

    // filter 2 stores the enable control IDs for brake, throttle, and steering
    // FIFO_1
    let mut f2 = CanFilterConfig::default();
    f2.filter_number = 2;
    f2.enabled = true;
    f2.mode = FilterMode::IdList;
    f2.fifo_assignment = RxFifo::Fifo1;
    f2.scale = FilterScale::Fs16Bit;
    f2.filter_mask_id_low = (OSCC_BRAKE_ENABLE_CAN_ID << 5) as _;
    f2.filter_id_low = (OSCC_THROTTLE_ENABLE_CAN_ID << 5) as _;
    f2.filter_mask_id_high = (OSCC_STEERING_ENABLE_CAN_ID << 5) as _;
    f2.filter_id_high = 0;

    [f0, f1, f2]
}

pub fn gather_obd_can_filters() -> [CanFilterConfig; 1] {
    // filter 14 stores the 4 OBD IDs
    // bank 14 means CAN2 treats this as the first filter
    // FIFO_0
    let mut f3 = CanFilterConfig::default();
    f3.filter_number = 14;
    f3.bank_number = 14;
    f3.enabled = true;
    f3.mode = FilterMode::IdList;
    f3.fifo_assignment = RxFifo::Fifo0;
    f3.scale = FilterScale::Fs16Bit;
    f3.filter_mask_id_low = (KIA_SOUL_OBD_STEERING_WHEEL_ANGLE_CAN_ID << 5) as _;
    f3.filter_id_low = (KIA_SOUL_OBD_WHEEL_SPEED_CAN_ID << 5) as _;
    f3.filter_mask_id_high = (KIA_SOUL_OBD_BRAKE_PRESSURE_CAN_ID << 5) as _;
    f3.filter_id_high = (KIA_SOUL_OBD_THROTTLE_PRESSURE_CAN_ID << 5) as _;

    [f3]
}
