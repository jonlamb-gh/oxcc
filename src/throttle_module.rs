// https://github.com/jonlamb-gh/oscc/tree/devel/firmware/throttle

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
use throttle_can_protocol::*;

// TODO feature gate vehicles
use kial_soul_ev::*;

// TODO - use some form of println! logging that prefixes with a module name?

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
    control_state: ThrottleControlState,
    grounded_fault_state: FaultCondition,
    operator_override_state: FaultCondition,
    throttle_report: OsccThrottleReport,
    fault_report_frame: OsccFaultReportFrame,
}

impl ThrottleModule {
    pub fn new() -> Self {
        ThrottleModule {
            accelerator_position: DualSignal::new(
                0,
                0,
                AdcSignal::AcceleratorPositionSensorHigh,
                AdcSignal::AcceleratorPositionSensorLow,
            ),
            control_state: ThrottleControlState::new(),
            grounded_fault_state: FaultCondition::new(),
            operator_override_state: FaultCondition::new(),
            throttle_report: OsccThrottleReport::new(),
            fault_report_frame: OsccFaultReportFrame::new(),
        }
    }

    pub fn init_devices(&self, board: &mut Board) {
        board.throttle_spoof_enable().set_low();
        // TODO - PIN_DAC_CHIP_SELECT, HIGH
    }

    pub fn disable_control(&mut self, board: &mut Board) {
        if self.control_state.enabled {
            self.accelerator_position
                .prevent_signal_discontinuity(board);

            board.throttle_spoof_enable().set_low();
            self.control_state.enabled = false;
            writeln!(board.debug_console, "Throttle control disabled");
        }
    }

    pub fn enable_control(&mut self, board: &mut Board) {
        if !self.control_state.enabled && !self.control_state.operator_override {
            self.accelerator_position
                .prevent_signal_discontinuity(board);

            board.throttle_spoof_enable().set_high();
            self.control_state.enabled = true;
            writeln!(board.debug_console, "Throttle control enabled");
        }
    }

    pub fn update_throttle(
        &mut self,
        spoof_command_high: u16,
        spoof_command_low: u16,
        board: &mut Board,
    ) {
        if self.control_state.enabled {
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

    pub fn check_for_faults(&mut self, board: &mut Board) {
        if self.control_state.enabled || self.control_state.dtcs > 0 {
            self.read_accelerator_position_sensor(board);

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

                self.control_state
                    .dtcs
                    .set(OSCC_THROTTLE_DTC_INVALID_SENSOR_VAL);

                self.publish_fault_report(board);

                writeln!(
                    board.debug_console,
                    "Bad value read from accelerator position sensor"
                );
            } else if operator_overridden {
                self.disable_control(board);

                self.control_state
                    .dtcs
                    .set(OSCC_THROTTLE_DTC_OPERATOR_OVERRIDE);

                self.publish_fault_report(board);

                writeln!(board.debug_console, "Throttle operator override");
            } else {
                self.control_state.dtcs = 0;
                self.control_state.operator_override = false;
            }
        }
    }

    pub fn publish_throttle_report(&mut self, board: &mut Board) {
        self.throttle_report.enabled = self.control_state.enabled;
        self.throttle_report.operator_override = self.control_state.operator_override;
        self.throttle_report.dtcs = self.control_state.dtcs;

        self.throttle_report.transmit(&mut board.control_can());
    }

    pub fn publish_fault_report(&mut self, board: &mut Board) {
        self.fault_report_frame.fault_report.fault_origin_id = FAULT_ORIGIN_THROTTLE;
        self.fault_report_frame.fault_report.dtcs = self.control_state.dtcs;

        self.fault_report_frame.transmit(&mut board.control_can());
    }

    // TODO - error handling
    pub fn process_rx_frame(&mut self, can_frame: &CanFrame, board: &mut Board) {
        if let CanFrame::DataFrame(ref frame) = can_frame {
            let id: u32 = frame.id().into();
            let data = frame.data();

            if (data[0] == OSCC_MAGIC_BYTE_0) && (data[1] == OSCC_MAGIC_BYTE_1) {
                if id == OSCC_THROTTLE_ENABLE_CAN_ID.into() {
                    self.enable_control(board);
                } else if id == OSCC_THROTTLE_DISABLE_CAN_ID.into() {
                    self.disable_control(board);
                } else if id == OSCC_THROTTLE_COMMAND_CAN_ID.into() {
                    self.process_throttle_command(&OsccThrottleCommand::from(frame), board);
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

    fn process_throttle_command(&mut self, command: &OsccThrottleCommand, board: &mut Board) {
        let clamped_position = num::clamp(
            command.torque_request,
            MINIMUM_THROTTLE_COMMAND,
            MAXIMUM_THROTTLE_COMMAND,
        );

        let spoof_voltage_low: f32 = num::clamp(
            self.throttle_position_to_volts_low(clamped_position),
            THROTTLE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN,
            THROTTLE_SPOOF_LOW_SIGNAL_VOLTAGE_MAX,
        );

        let spoof_voltage_high: f32 = num::clamp(
            self.throttle_position_to_volts_high(clamped_position),
            THROTTLE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN,
            THROTTLE_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX,
        );

        let spoof_value_low = (STEPS_PER_VOLT * spoof_voltage_low) as u16;
        let spoof_value_high = (STEPS_PER_VOLT * spoof_voltage_high) as u16;

        self.update_throttle(spoof_value_high, spoof_value_low, board);
    }

    fn throttle_position_to_volts_low(&self, pos: f32) -> f32 {
        pos * (THROTTLE_SPOOF_LOW_SIGNAL_VOLTAGE_MAX - THROTTLE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN)
            + THROTTLE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN
    }

    fn throttle_position_to_volts_high(&self, pos: f32) -> f32 {
        pos * (THROTTLE_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX - THROTTLE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN)
            + THROTTLE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN
    }

    fn read_accelerator_position_sensor(&mut self, board: &mut Board) {
        self.accelerator_position.update(board);
    }
}
