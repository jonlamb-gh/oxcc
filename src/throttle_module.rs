// https://github.com/jonlamb-gh/oscc/blob/master/firmware/throttle/src/main.cpp
// https://github.com/jonlamb-gh/oscc/blob/master/firmware/throttle/include/throttle_control.h
// https://github.com/jonlamb-gh/oscc/blob/master/firmware/throttle/src/throttle_control.cpp

use board::Board;
use core::fmt::Write;
use dual_signal::DualSignal;
use fault_can_protocol::*;
use fault_condition::FaultCondition;
use nucleo_f767zi::can::CanFrame;
use nucleo_f767zi::hal::prelude::*;
use num;
use throttle_can_protocol::*;

// TODO feature gate vehicles
use kial_soul_ev::*;

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
    accelerator_position: DualSignal,
    throttle_control_state: ThrottleControlState,
    grounded_fault_state: FaultCondition,
    operator_override_state: FaultCondition,
    throttle_report: OsccThrottleReport,
    fault_report: OsccFaultReport,
}

impl ThrottleModule {
    pub fn new() -> Self {
        ThrottleModule {
            accelerator_position: DualSignal::new(0, 0),
            throttle_control_state: ThrottleControlState::new(),
            grounded_fault_state: FaultCondition::new(),
            operator_override_state: FaultCondition::new(),
            throttle_report: OsccThrottleReport::new(),
            fault_report: OsccFaultReport::new(),
        }
    }

    pub fn disable_control(&mut self, board: &mut Board) {
        if self.throttle_control_state.enabled {
            board
                .dac
                .prevent_signal_discontinuity(&self.accelerator_position);

            board.throttle_spoof_enable.set_low();
            self.throttle_control_state.enabled = false;
            writeln!(board.debug_console, "Control disabled");
        }
    }

    pub fn enable_control(&mut self, board: &mut Board) {
        if !self.throttle_control_state.enabled && !self.throttle_control_state.operator_override {
            board
                .dac
                .prevent_signal_discontinuity(&self.accelerator_position);

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

            // TODO - revisit this, enforce high->A, low->B
            board.dac.set_outputs(spoof_high, spoof_low);
        }
    }

    // Normally via an interrupt handler.
    pub fn adc_input(&mut self, high: u16, low: u16) {
        self.accelerator_position.update(high, low);
    }

    pub fn check_for_faults(&mut self, board: &mut Board) {
        if self.throttle_control_state.enabled && self.throttle_control_state.dtcs > 0 {
            let accelerator_position_average = self.accelerator_position.average();

            let operator_overridden: bool =
                self.operator_override_state.condition_exceeded_duration(
                    accelerator_position_average >= ACCELERATOR_OVERRIDE_THRESHOLD,
                    FAULT_HYSTERESIS,
                    board,
                );

            let inputs_grounded: bool = self.grounded_fault_state.check_voltage_grounded(
                &self.accelerator_position,
                FAULT_HYSTERESIS,
                board,
            );

            // sensor pins tied to ground - a value of zero indicates disconnection
            if inputs_grounded {
                self.disable_control(board);

                // TODO
                // DTC get/set/etc
                /*
                 DTC_SET(
                    g_throttle_control_state.dtcs,
                    OSCC_THROTTLE_DTC_INVALID_SENSOR_VAL );
                */

                self.publish_fault_report(board);

                writeln!(
                    board.debug_console,
                    "Bad value read from accelerator position sensor"
                );
            } else if operator_overridden {
                self.disable_control(board);

                // TODO
                // DTC get/set/etc
                /*
                DTC_SET(
                    g_throttle_control_state.dtcs,
                    OSCC_THROTTLE_DTC_OPERATOR_OVERRIDE );
                */

                self.publish_fault_report(board);

                writeln!(board.debug_console, "Operator override");
            } else {
                self.throttle_control_state.dtcs = 0;

                if self.throttle_control_state.operator_override {
                    self.throttle_control_state.operator_override = false;
                }
            }
        }
    }

    pub fn publish_throttle_report(&mut self, board: &mut Board) {
        self.throttle_report.enabled = self.throttle_control_state.enabled;
        self.throttle_report.operator_override = self.throttle_control_state.operator_override;
        self.throttle_report.dtcs = self.throttle_control_state.dtcs;

        self.throttle_report.transmit(&mut board.control_can);
    }

    pub fn publish_fault_report(&mut self, board: &mut Board) {
        self.fault_report.fault_origin_id = FAULT_ORIGIN_THROTTLE;
        self.fault_report.dtcs = self.throttle_control_state.dtcs;

        self.fault_report.transmit(&mut board.control_can);
    }

    pub fn check_for_incoming_message(&mut self, board: &mut Board) {
        if let Ok(rx_frame) = board.control_can.receive() {
            self.process_rx_frame(&rx_frame);
        }
    }

    pub fn process_rx_frame(&mut self, frame: &CanFrame) {
        // TODO - need the CAN spec
    }
}
