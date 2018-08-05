// https://github.com/jonlamb-gh/oscc/blob/master/firmware/throttle/src/main.cpp
// https://github.com/jonlamb-gh/oscc/blob/master/firmware/throttle/include/throttle_control.h
// https://github.com/jonlamb-gh/oscc/blob/master/firmware/throttle/src/throttle_control.cpp

use board::Board;
use core::fmt::Write;
use fault_condition::FaultCondition;
use nucleo_f767zi::hal::prelude::*;

struct AcceleratorPosition {
    low: u16,
    high: u16,
}

impl AcceleratorPosition {
    pub const fn new() -> Self {
        AcceleratorPosition { low: 0, high: 0 }
    }
}

struct ThrottleControlState {
    enabled: bool,
    operator_override: bool,
    dtcs: u8,
}

impl ThrottleControlState {
    pub const fn new() -> Self {
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
    pub const fn new() -> Self {
        ThrottleModule {
            accelerator_position: AcceleratorPosition::new(),
            throttle_control_state: ThrottleControlState::new(),
            grounded_fault_state: FaultCondition::new(),
            operator_override_state: FaultCondition::new(),
        }
    }

    pub fn disable_control(&mut self, board: &mut Board) {
        if self.throttle_control_state.enabled {
            // TODO
            //prevent_signal_disc...

            board.throttle_spoof_enable.set_low();
            self.throttle_control_state.enabled = false;
            writeln!(board.debug_console, "Control disabled");
        }
    }

    pub fn enable_control(&mut self, board: &mut Board) {
        if !self.throttle_control_state.enabled && !self.throttle_control_state.operator_override {
            // TODO
            //prevent_signal_disc...

            board.throttle_spoof_enable.set_high();
            self.throttle_control_state.enabled = true;
            writeln!(board.debug_console, "Control enabled");
        }
    }

    pub fn update_throttle(&mut self, spoof_command_high: u16, spoof_command_low: u16) {
        if self.throttle_control_state.enabled {
            // TODO
            // DAC output
        }
    }

    // Normally via an interrupt handler.
    pub fn adc_input(&mut self, high: u16, low: u16) {
        self.accelerator_position.high = high;
        self.accelerator_position.low = low;
    }

    pub fn check_for_faults(&mut self, board: &mut Board) {
        // TODO
    }
}
