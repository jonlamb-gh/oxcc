//! OxCC configuration data

use brake_can_protocol::*;
use fault_can_protocol::*;
use nucleo_f767zi::hal::can::{
    CanBitTiming, CanConfig, CanFilterConfig, FilterMode, FilterScale, RxFifo,
};
use steering_can_protocol::*;
use throttle_can_protocol::*;
use vehicle::*;

/// Control CAN interface configuration
pub const CONTROL_CAN_CONFIG: CanConfig = CanConfig {
    loopback_mode: false,
    silent_mode: false,
    ttcm: false,
    abom: true,
    awum: false,
    nart: false,
    rflm: false,
    txfp: false,
    // TODO - update CAN impl to calculate these
    /// Control CAN bus is configured for 500K
    bit_timing: CanBitTiming {
        prescaler: 5, // 6
        sjw: 0,       // CAN_SJW_1TQ
        bs1: 14,      // CAN_BS1_15TQ
        bs2: 1,       // CAN_BS2_2TQ
    },
};

/// Vehicle OBD CAN interface configuration
pub const OBD_CAN_CONFIG: CanConfig = CanConfig {
    loopback_mode: false,
    silent_mode: false,
    ttcm: false,
    abom: true,
    awum: false,
    nart: false,
    rflm: false,
    txfp: false,
    /// OBD CAN bus is configured for 500K
    bit_timing: CanBitTiming {
        prescaler: 5, // 6
        sjw: 0,       // CAN_SJW_1TQ
        bs1: 14,      // CAN_BS1_15TQ
        bs2: 1,       // CAN_BS2_2TQ
    },
};

/// Gather the Control CAN filter configurations
///
/// Since we're only interrested in a small number of messages,
/// we can use ID list mode instead of masking.
/// Only the specific message IDs are allowed through the filter.
/// Filter 0 is the highest priority filter, followed by filter 1, etc.
pub fn gather_control_can_filters() -> [CanFilterConfig; 3] {
    // Filter 0, bound to FIFO_0
    //  - disable control IDs for throttle, brake, steering
    //  - fault report ID
    let mut f0 = CanFilterConfig::default();
    f0.filter_number = 0;
    f0.enabled = true;
    f0.mode = FilterMode::IdList;
    f0.fifo_assignment = RxFifo::Fifo0;
    f0.scale = FilterScale::Fs16Bit;
    f0.filter_mask_id_low = u32::from(OSCC_THROTTLE_DISABLE_CAN_ID << 5);
    f0.filter_id_low = u32::from(OSCC_BRAKE_DISABLE_CAN_ID << 5);
    f0.filter_mask_id_high = u32::from(OSCC_STEERING_DISABLE_CAN_ID << 5);
    f0.filter_id_high = u32::from(OSCC_FAULT_REPORT_CAN_ID << 5);

    // Filter 1, bound to FIFO_1
    // - control command IDs for brake, throttle, and steering
    let mut f1 = CanFilterConfig::default();
    f1.filter_number = 1;
    f1.enabled = true;
    f1.mode = FilterMode::IdList;
    f1.fifo_assignment = RxFifo::Fifo1;
    f1.scale = FilterScale::Fs16Bit;
    f1.filter_mask_id_low = u32::from(OSCC_BRAKE_COMMAND_CAN_ID << 5);
    f1.filter_id_low = u32::from(OSCC_THROTTLE_COMMAND_CAN_ID << 5);
    f1.filter_mask_id_high = u32::from(OSCC_STEERING_COMMAND_CAN_ID << 5);
    f1.filter_id_high = 0;

    // filter 2, bound to FIFO_1
    // - enable control IDs for brake, throttle, and steering
    let mut f2 = CanFilterConfig::default();
    f2.filter_number = 2;
    f2.enabled = true;
    f2.mode = FilterMode::IdList;
    f2.fifo_assignment = RxFifo::Fifo1;
    f2.scale = FilterScale::Fs16Bit;
    f2.filter_mask_id_low = u32::from(OSCC_BRAKE_ENABLE_CAN_ID << 5);
    f2.filter_id_low = u32::from(OSCC_THROTTLE_ENABLE_CAN_ID << 5);
    f2.filter_mask_id_high = u32::from(OSCC_STEERING_ENABLE_CAN_ID << 5);
    f2.filter_id_high = 0;

    [f0, f1, f2]
}

/// Gather the vehicle OBD CAN filter configurations
///
/// **NOTE**
/// Vehicle OBD CAN is on CAN2 which shares a configuration
/// peripheral with CAN1 (see notes on bxCAN). The split is 50/50, so
/// CAN1 gets the first 0:13 filters and CAN2 gets the remaining 14:27
/// filters. This is why the filter numbers here start at 14.
///
/// Since we're only interrested in a small number of messages,
/// we can use ID list mode instead of masking.
/// Only the specific message IDs are allowed through the filter.
/// Filter 0 is the highest priority filter, followed by filter 1, etc.
pub fn gather_obd_can_filters() -> [CanFilterConfig; 1] {
    // filter 14, bound to FIFO_0
    // - the 4 OBD IDs
    let mut f3 = CanFilterConfig::default();
    f3.filter_number = 14;
    f3.bank_number = 14;
    f3.enabled = true;
    f3.mode = FilterMode::IdList;
    f3.fifo_assignment = RxFifo::Fifo0;
    f3.scale = FilterScale::Fs16Bit;
    f3.filter_mask_id_low = u32::from(KIA_SOUL_OBD_STEERING_WHEEL_ANGLE_CAN_ID << 5);
    f3.filter_id_low = u32::from(KIA_SOUL_OBD_WHEEL_SPEED_CAN_ID << 5);
    f3.filter_mask_id_high = u32::from(KIA_SOUL_OBD_BRAKE_PRESSURE_CAN_ID << 5);
    #[cfg(feature = "kia-soul-ev")]
    {
        f3.filter_id_high = u32::from(KIA_SOUL_OBD_THROTTLE_PRESSURE_CAN_ID << 5);
    }

    [f3]
}
