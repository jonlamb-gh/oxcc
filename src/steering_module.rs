// https://github.com/jonlamb-gh/oscc/tree/devel/firmware/steering

use adc_signal::AdcSignal;
use board::Board;
use core::fmt::Write;
use dtc::DtcBitfield;
use dual_signal::DualSignal;
use fault_can_protocol::*;
use fault_condition::FaultCondition;
use nucleo_f767zi::hal::can::CanFrame;
use nucleo_f767zi::hal::prelude::*;
use num;
use oscc_magic_byte::*;
use steering_can_protocol::*;
use vehicle::*;

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
    steering_report: OsccSteeringReport,
    fault_report_frame: OsccFaultReportFrame,
}

impl SteeringModule {
    pub fn new() -> Self {
        SteeringModule {
            steering_torque: DualSignal::new(
                0,
                0,
                AdcSignal::TorqueSensorHigh,
                AdcSignal::TorqueSensorLow,
            ),
            control_state: SteeringControlState::new(),
            grounded_fault_state: FaultCondition::new(),
            filtered_diff: 0,
            steering_report: OsccSteeringReport::new(),
            fault_report_frame: OsccFaultReportFrame::new(),
        }
    }

    pub fn init_devices(&self, board: &mut Board) {
        board.steering_spoof_enable().set_low();
        // TODO - PIN_DAC_CHIP_SELECT, HIGH
    }

    pub fn disable_control(&mut self, board: &mut Board) {
        if self.control_state.enabled {
            self.steering_torque.prevent_signal_discontinuity(board);

            board.steering_spoof_enable().set_low();
            self.control_state.enabled = false;
            writeln!(board.debug_console, "Steering control disabled");
        }
    }

    pub fn enable_control(&mut self, board: &mut Board) {
        if !self.control_state.enabled && !self.control_state.operator_override {
            self.steering_torque.prevent_signal_discontinuity(board);

            board.steering_spoof_enable().set_high();
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

    pub fn check_for_faults(&mut self, board: &mut Board) {
        if self.control_state.enabled || self.control_state.dtcs > 0 {
            self.read_torque_sensor(board);

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

    pub fn publish_steering_report(&mut self, board: &mut Board) {
        self.steering_report.enabled = self.control_state.enabled;
        self.steering_report.operator_override = self.control_state.operator_override;
        self.steering_report.dtcs = self.control_state.dtcs;

        self.steering_report.transmit(&mut board.control_can());
    }

    pub fn publish_fault_report(&mut self, board: &mut Board) {
        self.fault_report_frame.fault_report.fault_origin_id = FAULT_ORIGIN_STEERING;
        self.fault_report_frame.fault_report.dtcs = self.control_state.dtcs;

        self.fault_report_frame.transmit(&mut board.control_can());
    }

    // TODO - error handling
    pub fn process_rx_frame(&mut self, can_frame: &CanFrame, board: &mut Board) {
        if let CanFrame::DataFrame(ref frame) = can_frame {
            let id: u32 = frame.id().into();
            let data = frame.data();

            if (data[0] == OSCC_MAGIC_BYTE_0) && (data[1] == OSCC_MAGIC_BYTE_1) {
                if id == OSCC_STEERING_ENABLE_CAN_ID.into() {
                    self.enable_control(board);
                } else if id == OSCC_STEERING_DISABLE_CAN_ID.into() {
                    self.disable_control(board);
                } else if id == OSCC_STEERING_COMMAND_CAN_ID.into() {
                    self.process_steering_command(&OsccSteeringCommand::from(frame), board);
                } else if id == OSCC_FAULT_REPORT_CAN_ID.into() {
                    self.process_fault_report(&OsccFaultReport::from(frame), board);
                }
            }
        }
    }

    fn process_fault_report(&mut self, fault_report: &OsccFaultReport, board: &mut Board) {
        self.disable_control(board);

        writeln!(
            board.debug_console,
            "Fault report received from: {} DTCs: {}",
            fault_report.fault_origin_id, fault_report.dtcs
        );
    }

    fn process_steering_command(&mut self, command: &OsccSteeringCommand, board: &mut Board) {
        let clamped_torque = num::clamp(
            command.torque_request * MAXIMUM_TORQUE_COMMAND,
            MINIMUM_TORQUE_COMMAND,
            MAXIMUM_TORQUE_COMMAND,
        );

        let spoof_voltage_low: f32 = num::clamp(
            self.steering_torque_to_volts_low(clamped_torque),
            STEERING_SPOOF_LOW_SIGNAL_VOLTAGE_MIN,
            STEERING_SPOOF_LOW_SIGNAL_VOLTAGE_MAX,
        );

        let spoof_voltage_high: f32 = num::clamp(
            self.steering_torque_to_volts_high(clamped_torque),
            STEERING_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN,
            STEERING_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX,
        );

        let spoof_value_low = (STEPS_PER_VOLT * spoof_voltage_low) as u16;
        let spoof_value_high = (STEPS_PER_VOLT * spoof_voltage_high) as u16;

        self.update_steering(spoof_value_high, spoof_value_low, board);
    }

    fn steering_torque_to_volts_low(&self, torque: f32) -> f32 {
        (TORQUE_SPOOF_LOW_SIGNAL_CALIBRATION_CURVE_SCALE * torque)
            + TORQUE_SPOOF_LOW_SIGNAL_CALIBRATION_CURVE_OFFSET
    }

    fn steering_torque_to_volts_high(&self, torque: f32) -> f32 {
        (TORQUE_SPOOF_HIGH_SIGNAL_CALIBRATION_CURVE_SCALE * torque)
            + TORQUE_SPOOF_HIGH_SIGNAL_CALIBRATION_CURVE_OFFSET
    }

    fn read_torque_sensor(&mut self, board: &mut Board) {
        self.steering_torque.update(board);
    }
}
