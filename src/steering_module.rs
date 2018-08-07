// https://github.com/jonlamb-gh/oscc/tree/master/firmware/steering

use board::Board;
use core::fmt::Write;
use dtc::DtcBitfield;
use dual_signal::DualSignal;
use fault_can_protocol::*;
use fault_condition::FaultCondition;
use nucleo_f767zi::can::CanFrame;
use nucleo_f767zi::hal::prelude::*;
use num;
use steering_can_protocol::*;

// TODO feature gate vehicles
use kial_soul_ev::*;

// TODO - use some form of println! logging that prefixes with a module name?

const FILTER_ALPHA: f32 = 0.01_f32;

struct SteeringControlState {
    enabled: bool,
    operator_override: bool,
    dtcs: u8,
}

impl SteeringControlState {
    pub const fn new() -> Self {
        SteeringControlState {
            enabled: false,
            operator_override: false,
            dtcs: 0,
        }
    }
}

pub struct SteeringModule {
    steering_torque: DualSignal,
    control_state: SteeringControlState,
    grounded_fault_state: FaultCondition,
    filtered_diff: u16,
    /* throttle_report: OsccThrottleReport, */
    /* fault_report_frame: OsccFaultReportFrame, */
}

impl SteeringModule {
    pub fn new() -> Self {
        SteeringModule {
            steering_torque: DualSignal::new(0, 0),
            control_state: SteeringControlState::new(),
            grounded_fault_state: FaultCondition::new(),
            filtered_diff: 0,
            /* throttle_report: OsccThrottleReport::new(), */
            /* fault_report_frame: OsccFaultReportFrame::new(), */
        }
    }

    pub fn init_devices(&self, board: &mut Board) {
        board.steering_spoof_enable.set_low();
        // TODO - PIN_DAC_CHIP_SELECT, HIGH
    }

    pub fn disable_control(&mut self, board: &mut Board) {
        if self.control_state.enabled {
            board
                .dac
                .prevent_signal_discontinuity(&self.steering_torque);

            board.steering_spoof_enable.set_low();
            self.control_state.enabled = false;
            writeln!(board.debug_console, "Steering control disabled");
        }
    }

    pub fn enable_control(&mut self, board: &mut Board) {
        if !self.control_state.enabled && !self.control_state.operator_override {
            board
                .dac
                .prevent_signal_discontinuity(&self.steering_torque);

            board.steering_spoof_enable.set_high();
            self.control_state.enabled = true;
            writeln!(board.debug_console, "Steering control enabled");
        }
    }

    pub fn update_steering(
        &mut self,
        spoof_command_high: u16,
        spoof_command_low: u16,
        board: &mut Board,
    ) {
        if self.control_state.enabled {
            let spoof_high = num::clamp(
                spoof_command_high,
                STEERING_SPOOF_HIGH_SIGNAL_RANGE_MIN,
                STEERING_SPOOF_HIGH_SIGNAL_RANGE_MAX,
            );

            let spoof_low = num::clamp(
                spoof_command_low,
                STEERING_SPOOF_LOW_SIGNAL_RANGE_MIN,
                STEERING_SPOOF_LOW_SIGNAL_RANGE_MAX,
            );

            // TODO - revisit this, enforce high->A, low->B
            board.dac.set_outputs(spoof_high, spoof_low);
        }
    }

    pub fn adc_input(&mut self, high: u16, low: u16) {
        self.steering_torque.update(high, low);
    }

    pub fn check_for_faults(&mut self, board: &mut Board) {
        if self.control_state.enabled && self.control_state.dtcs > 0 {
            let unfiltered_diff = self.steering_torque.diff();

            if self.filtered_diff == 0 {
                self.filtered_diff = unfiltered_diff;
            }

            // TODO - revist this
            // OSCC goes back and forth with u16 and f32 types
            self.filtered_diff = self.exponential_moving_average(
                FILTER_ALPHA,
                unfiltered_diff as _,
                self.filtered_diff as _,
            ) as _;

            let inputs_grounded: bool = self.grounded_fault_state.check_voltage_grounded(
                &self.steering_torque,
                FAULT_HYSTERESIS,
                board,
            );

            // sensor pins tied to ground - a value of zero indicates disconnection
            if inputs_grounded {
                self.disable_control(board);

                self.control_state
                    .dtcs
                    .set(OSCC_STEERING_DTC_INVALID_SENSOR_VAL);

                self.publish_fault_report(board);

                writeln!(board.debug_console, "Bad value read from torque sensor");
            } else if self.filtered_diff > TORQUE_DIFFERENCE_OVERRIDE_THRESHOLD {
                self.disable_control(board);

                self.control_state
                    .dtcs
                    .set(OSCC_STEERING_DTC_OPERATOR_OVERRIDE);

                self.publish_fault_report(board);

                self.control_state.operator_override = true;

                writeln!(board.debug_console, "Steering operator override");
            } else {
                self.control_state.dtcs = 0;
                self.control_state.operator_override = false;
            }
        }
    }

    fn exponential_moving_average(&self, alpha: f32, input: f32, average: f32) -> f32 {
        (alpha * input) + ((1.0 - alpha) * average)
    }

    pub fn publish_steering_report(&mut self, board: &mut Board) {}

    pub fn publish_fault_report(&mut self, board: &mut Board) {}

    pub fn check_for_incoming_message(&mut self, board: &mut Board) {}

    pub fn process_rx_frame(&mut self, frame: &CanFrame, board: &mut Board) {}
}
