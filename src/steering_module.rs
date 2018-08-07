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
//use steering_can_protocol::*;

// TODO feature gate vehicles
use kial_soul_ev::*;

// TODO - use some form of println! logging that prefixes with a module name?

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
    /*throttle_report: OsccThrottleReport,
     *fault_report_frame: OsccFaultReportFrame, */
}

impl SteeringModule {
    pub fn new() -> Self {
        SteeringModule {
            steering_torque: DualSignal::new(0, 0),
            control_state: SteeringControlState::new(),
            grounded_fault_state: FaultCondition::new(),
            /*throttle_report: OsccThrottleReport::new(),
             *fault_report_frame: OsccFaultReportFrame::new(), */
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

    // Normally via an interrupt handler.
    pub fn adc_input(&mut self, high: u16, low: u16) {}

    pub fn check_for_faults(&mut self, board: &mut Board) {}

    pub fn publish_steering_report(&mut self, board: &mut Board) {}

    pub fn publish_fault_report(&mut self, board: &mut Board) {}

    pub fn check_for_incoming_message(&mut self, board: &mut Board) {}

    pub fn process_rx_frame(&mut self, frame: &CanFrame, board: &mut Board) {}
}
