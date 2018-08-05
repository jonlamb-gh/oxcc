// https://github.com/jonlamb-gh/oscc/blob/master/firmware/throttle/src/main.cpp
// https://github.com/jonlamb-gh/oscc/blob/master/firmware/throttle/include/throttle_control.h
// https://github.com/jonlamb-gh/oscc/blob/master/firmware/throttle/src/throttle_control.cpp

use board::Board;
use core::fmt::Write;
use fault_condition::FaultCondition;
use nucleo_f767zi::hal::prelude::*;
use num;

// TODO feature gate vehicles
use kial_soul_ev::*;

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
            // TODO - revist this
            board.dac.prevent_signal_discontinuity(
                self.accelerator_position.low,
                self.accelerator_position.high,
            );

            board.throttle_spoof_enable.set_low();
            self.throttle_control_state.enabled = false;
            writeln!(board.debug_console, "Control disabled");
        }
    }

    pub fn enable_control(&mut self, board: &mut Board) {
        if !self.throttle_control_state.enabled && !self.throttle_control_state.operator_override {
            // TODO - revist this
            board.dac.prevent_signal_discontinuity(
                self.accelerator_position.low,
                self.accelerator_position.high,
            );

            board.throttle_spoof_enable.set_high();
            self.throttle_control_state.enabled = true;
            writeln!(board.debug_console, "Control enabled");
        }
    }

    pub fn update_throttle(
        &mut self,
        spoof_command_high: u16,
        spoof_command_low: u16,
        board: &mut Board,
    ) {
        if self.throttle_control_state.enabled {
            let spoof_high = num::clamp(
                spoof_command_high,
                THROTTLE_SPOOF_HIGH_SIGNAL_RANGE_MIN,
                THROTTLE_SPOOF_HIGH_SIGNAL_RANGE_MAX,
            );

            let spoof_low = num::clamp(
                spoof_command_low,
                THROTTLE_SPOOF_LOW_SIGNAL_RANGE_MIN,
                THROTTLE_SPOOF_LOW_SIGNAL_RANGE_MAX,
            );

            board.dac.set_outputs(spoof_high, spoof_low);
        }
    }

    // Normally via an interrupt handler.
    pub fn adc_input(&mut self, high: u16, low: u16) {
        self.accelerator_position.high = high;
        self.accelerator_position.low = low;
    }

    pub fn check_for_faults(&mut self, board: &mut Board) {
        if self.throttle_control_state.enabled && self.throttle_control_state.dtcs > 0 {
            let accelerator_position_average: u32 =
                (self.accelerator_position.low as u32 + self.accelerator_position.high as u32) / 2;

            let operator_overridden: bool =
                self.operator_override_state.condition_exceeded_duration(
                    accelerator_position_average >= ACCELERATOR_OVERRIDE_THRESHOLD,
                    FAULT_HYSTERESIS,
                    board,
                );

            let inputs_grounded: bool = self.grounded_fault_state.check_voltage_grounded(
                self.accelerator_position.high,
                self.accelerator_position.low,
                FAULT_HYSTERESIS,
                board,
            );

            // sensor pins tied to ground - a value of zero indicates disconnection
            if inputs_grounded {
                self.disable_control(board);

                // TODO
                // DTC get/set/etc

                // TODO
                // CAN comms

                writeln!(
                    board.debug_console,
                    "Bad value read from accelerator position sensor"
                );
            } else if operator_overridden {
                self.disable_control(board);

                // TODO
                // DTC get/set/etc

                // TODO
                // CAN comms

                writeln!(board.debug_console, "Operator override");
            } else {
                self.throttle_control_state.dtcs = 0;

                if self.throttle_control_state.operator_override {
                    self.throttle_control_state.operator_override = false;
                }
            }
        }
    }
}
