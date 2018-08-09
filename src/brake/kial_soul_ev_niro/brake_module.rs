// https://github.com/jonlamb-gh/oscc/tree/devel/firmware/brake/kia_soul_ev_niro

use board::Board;
use brake_can_protocol::*;
use core::fmt::Write;
use dtc::DtcBitfield;
use dual_signal::DualSignal;
use fault_can_protocol::*;
use fault_condition::FaultCondition;
use kial_soul_ev::*;
use nucleo_f767zi::can::{CanFrame, DataFrame};
use nucleo_f767zi::hal::prelude::*;
use num;
use oscc_magic_byte::*;

// TODO - use some form of println! logging that prefixes with a module name?

struct BrakeControlState {
    enabled: bool,
    operator_override: bool,
    dtcs: u8,
}

impl BrakeControlState {
    pub const fn new() -> Self {
        BrakeControlState {
            enabled: false,
            operator_override: false,
            dtcs: 0,
        }
    }
}

pub struct BrakeModule {
    brake_pedal_position: DualSignal,
    control_state: BrakeControlState,
    grounded_fault_state: FaultCondition,
    operator_override_state: FaultCondition,
    brake_report: OsccBrakeReport,
    fault_report_frame: OsccFaultReportFrame,
}

impl BrakeModule {
    pub fn new() -> Self {
        BrakeModule {
            brake_pedal_position: DualSignal::new(0, 0),
            control_state: BrakeControlState::new(),
            grounded_fault_state: FaultCondition::new(),
            operator_override_state: FaultCondition::new(),
            brake_report: OsccBrakeReport::new(),
            fault_report_frame: OsccFaultReportFrame::new(),
        }
    }

    pub fn init_devices(&self, board: &mut Board) {
        board.brake_spoof_enable().set_low();
        board.brake_light_enable().set_low();
        // TODO - PIN_DAC_CHIP_SELECT, HIGH
    }

    pub fn disable_control(&mut self, board: &mut Board) {
        if self.control_state.enabled {
            board
                .dac
                .prevent_signal_discontinuity(&self.brake_pedal_position);

            board.brake_spoof_enable().set_low();
            board.brake_light_enable().set_low();
            self.control_state.enabled = false;
            writeln!(board.debug_console, "Brake control disabled");
        }
    }

    pub fn enable_control(&mut self, board: &mut Board) {
        if !self.control_state.enabled && !self.control_state.operator_override {
            board
                .dac
                .prevent_signal_discontinuity(&self.brake_pedal_position);

            board.brake_spoof_enable().set_high();
            self.control_state.enabled = true;
            writeln!(board.debug_console, "Brake control enabled");
        }
    }

    pub fn update_brake(
        &mut self,
        spoof_command_high: u16,
        spoof_command_low: u16,
        board: &mut Board,
    ) {
        if self.control_state.enabled {
            let spoof_high = num::clamp(
                spoof_command_high,
                BRAKE_SPOOF_HIGH_SIGNAL_RANGE_MIN,
                BRAKE_SPOOF_HIGH_SIGNAL_RANGE_MAX,
            );

            let spoof_low = num::clamp(
                spoof_command_low,
                BRAKE_SPOOF_LOW_SIGNAL_RANGE_MIN,
                BRAKE_SPOOF_LOW_SIGNAL_RANGE_MAX,
            );

            if (spoof_high > BRAKE_LIGHT_SPOOF_HIGH_THRESHOLD)
                || (spoof_low > BRAKE_LIGHT_SPOOF_LOW_THRESHOLD)
            {
                board.brake_light_enable().set_high();
            } else {
                board.brake_light_enable().set_low();
            }

            // TODO - revisit this, enforce high->A, low->B
            board.dac.set_outputs(spoof_high, spoof_low);
        }
    }

    pub fn adc_input(&mut self, high: u16, low: u16) {
        self.brake_pedal_position.update(high, low);
    }

    pub fn check_for_faults(&mut self, board: &mut Board) {
        if self.control_state.enabled && self.control_state.dtcs > 0 {
            let brake_pedal_position_average = self.brake_pedal_position.average();

            let operator_overridden: bool =
                self.operator_override_state.condition_exceeded_duration(
                    brake_pedal_position_average >= BRAKE_PEDAL_OVERRIDE_THRESHOLD.into(),
                    FAULT_HYSTERESIS,
                    board,
                );

            let inputs_grounded: bool = self.grounded_fault_state.check_voltage_grounded(
                &self.brake_pedal_position,
                FAULT_HYSTERESIS,
                board,
            );

            // sensor pins tied to ground - a value of zero indicates disconnection
            if inputs_grounded {
                self.disable_control(board);

                self.control_state
                    .dtcs
                    .set(OSCC_BRAKE_DTC_INVALID_SENSOR_VAL);

                self.publish_fault_report(board);

                writeln!(
                    board.debug_console,
                    "Bad value read from brake pedal position sensor"
                );
            } else if operator_overridden {
                self.disable_control(board);

                self.control_state
                    .dtcs
                    .set(OSCC_BRAKE_DTC_OPERATOR_OVERRIDE);

                self.publish_fault_report(board);

                writeln!(board.debug_console, "Brake operator override");
            } else {
                self.control_state.dtcs = 0;
                self.control_state.operator_override = false;
            }
        }
    }

    pub fn publish_brake_report(&mut self, board: &mut Board) {
        self.brake_report.enabled = self.control_state.enabled;
        self.brake_report.operator_override = self.control_state.operator_override;
        self.brake_report.dtcs = self.control_state.dtcs;

        self.brake_report.transmit(&mut board.control_can());
    }

    pub fn publish_fault_report(&mut self, board: &mut Board) {
        self.fault_report_frame.fault_report.fault_origin_id = FAULT_ORIGIN_BRAKE;
        self.fault_report_frame.fault_report.dtcs = self.control_state.dtcs;

        self.fault_report_frame.transmit(&mut board.control_can());
    }

    // TODO - error handling
    pub fn check_for_incoming_message(&mut self, board: &mut Board) {
        if let Ok(rx_frame) = board.control_can().receive() {
            if let CanFrame::DataFrame(ref f) = rx_frame {
                self.process_rx_frame(f, board);
            }
        }
    }

    // TODO - error handling
    pub fn process_rx_frame(&mut self, frame: &DataFrame, board: &mut Board) {
        let id: u32 = frame.id().into();
        let data = frame.data();

        assert_eq!(data[0], OSCC_MAGIC_BYTE_0);
        assert_eq!(data[1], OSCC_MAGIC_BYTE_1);

        if id == OSCC_BRAKE_ENABLE_CAN_ID.into() {
            self.enable_control(board);
        } else if id == OSCC_BRAKE_DISABLE_CAN_ID.into() {
            self.disable_control(board);
        } else if id == OSCC_BRAKE_COMMAND_CAN_ID.into() {
            self.process_brake_command(&OsccBrakeCommand::from(frame), board);
        } else if id == OSCC_FAULT_REPORT_CAN_ID.into() {
            self.process_fault_report(&OsccFaultReport::from(frame), board);
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

    fn process_brake_command(&mut self, command: &OsccBrakeCommand, board: &mut Board) {
        let clamped_position = num::clamp(
            command.pedal_command,
            MINIMUM_BRAKE_COMMAND,
            MAXIMUM_BRAKE_COMMAND,
        );

        let spoof_voltage_low: f32 = num::clamp(
            self.brake_position_to_volts_low(clamped_position),
            BRAKE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN,
            BRAKE_SPOOF_LOW_SIGNAL_VOLTAGE_MAX,
        );

        let spoof_voltage_high: f32 = num::clamp(
            self.brake_position_to_volts_high(clamped_position),
            BRAKE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN,
            BRAKE_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX,
        );

        let spoof_value_low = (STEPS_PER_VOLT * spoof_voltage_low) as u16;
        let spoof_value_high = (STEPS_PER_VOLT * spoof_voltage_high) as u16;

        self.update_brake(spoof_value_high, spoof_value_low, board);
    }

    fn brake_position_to_volts_low(&self, pos: f32) -> f32 {
        pos * (BRAKE_SPOOF_LOW_SIGNAL_VOLTAGE_MAX - BRAKE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN)
            + BRAKE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN
    }

    fn brake_position_to_volts_high(&self, pos: f32) -> f32 {
        pos * (BRAKE_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX - BRAKE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN)
            + BRAKE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN
    }
}
