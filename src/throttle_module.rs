// https://github.com/jonlamb-gh/oscc/blob/master/firmware/throttle/src/main.cpp
// https://github.com/jonlamb-gh/oscc/blob/master/firmware/throttle/include/throttle_control.h
// https://github.com/jonlamb-gh/oscc/blob/master/firmware/throttle/src/throttle_control.cpp

use fault_condition::FaultCondition;

struct AcceleratorPosition {
    low: u16,
    high: u16,
}

impl AcceleratorPosition {
    pub fn new() -> AcceleratorPosition {
        AcceleratorPosition { low: 0, high: 0 }
    }
}

struct ThrottleControlState {
    enabled: bool,
    operator_override: bool,
    dtcs: u8,
}

impl ThrottleControlState {
    pub fn new() -> ThrottleControlState {
        ThrottleControlState {
            enabled: false,
            operator_override: false,
            dtcs: 0,
        }
    }
}

pub struct ThrottleModule {
    accelerator_position: AcceleratorPosition,
    throttle_control_state: ThrottleControlState,
    grounded_fault_state: FaultCondition,
    operator_override_state: FaultCondition,
}

impl ThrottleModule {
    pub fn new() -> ThrottleModule {
        ThrottleModule {
            accelerator_position: AcceleratorPosition::new(),
            throttle_control_state: ThrottleControlState::new(),
            grounded_fault_state: FaultCondition::new(),
            operator_override_state: FaultCondition::new(),
        }
    }

    // TODO
    pub fn disable_control(&mut self) {
        //
    }

    // TODO - need ADC
    //fn read_accelerator_position_sensor(&self) -> AcceleratorPosition {
}
