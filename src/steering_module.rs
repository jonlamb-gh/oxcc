// https://github.com/jonlamb-gh/oscc/tree/devel/firmware/steering

use board::TorqueSensor;
use core::fmt::Write;
use dac_mcp4922::DacOutput;
use dtc::DtcBitfield;
use dual_signal::DualSignal;
use fault_can_protocol::*;
use fault_condition::FaultCondition;
use ms_timer::MsTimer;
use nucleo_f767zi::debug_console::DebugConsole;
use nucleo_f767zi::hal::can::CanFrame;
use nucleo_f767zi::hal::prelude::*;
use num;
use oscc_magic_byte::*;
use ranges;
use steering_can_protocol::*;
use types::*;
use vehicle::*;

const FILTER_ALPHA: f32 = 0.01_f32;

struct SteeringControlState<DTCS: DtcBitfield> {
    enabled: bool,
    operator_override: bool,
    dtcs: DTCS,
}

impl<DTCS> SteeringControlState<DTCS>
where
    DTCS: DtcBitfield,
{
    pub const fn new(dtcs: DTCS) -> Self {
        SteeringControlState {
            enabled: false,
            operator_override: false,
            dtcs,
        }
    }
}

pub struct SteeringModule {
    steering_torque: DualSignal<TorqueSensor>,
    control_state: SteeringControlState<u8>,
    grounded_fault_state: FaultCondition,
    filtered_diff: u16,
    steering_report: OsccSteeringReport,
    fault_report: OsccFaultReport,
    steering_dac: SteeringDac,
    steering_pins: SteeringPins,
}

pub struct UnpreparedSteeringModule {
    steering_module: SteeringModule,
}

impl UnpreparedSteeringModule {
    pub fn new(
        torque_sensor: TorqueSensor,
        steering_dac: SteeringDac,
        steering_pins: SteeringPins,
    ) -> Self {
        UnpreparedSteeringModule {
            steering_module: SteeringModule {
                steering_torque: DualSignal::new(0, 0, torque_sensor),
                control_state: SteeringControlState::new(u8::default()),
                grounded_fault_state: FaultCondition::new(),
                filtered_diff: 0,
                steering_report: OsccSteeringReport::new(),
                fault_report: OsccFaultReport {
                    fault_origin_id: FAULT_ORIGIN_STEERING,
                    dtcs: 0,
                },
                steering_dac,
                steering_pins,
            },
        }
    }

    pub fn prepare_module(self) -> SteeringModule {
        let mut steering_module = self.steering_module;
        steering_module.steering_pins.spoof_enable.set_low();
        steering_module
    }
}

impl SteeringModule {
    pub fn disable_control(&mut self, debug_console: &mut DebugConsole) {
        if self.control_state.enabled {
            self.steering_torque.prevent_signal_discontinuity();

            self.steering_dac.output_ab(
                DacOutput::clamp(self.steering_torque.low()),
                DacOutput::clamp(self.steering_torque.high()),
            );

            self.steering_pins.spoof_enable.set_low();
            self.control_state.enabled = false;
            writeln!(debug_console, "Steering control disabled");
        }
    }

    pub fn enable_control(&mut self, debug_console: &mut DebugConsole) {
        if !self.control_state.enabled && !self.control_state.operator_override {
            self.steering_torque.prevent_signal_discontinuity();

            self.steering_dac.output_ab(
                DacOutput::clamp(self.steering_torque.low()),
                DacOutput::clamp(self.steering_torque.high()),
            );

            self.steering_pins.spoof_enable.set_high();
            self.control_state.enabled = true;
            writeln!(debug_console, "Steering control enabled");
        }
    }

    pub fn update_steering(&mut self, spoof_command_high: u16, spoof_command_low: u16) {
        if self.control_state.enabled {
            // TODO - revisit this, enforce high->A, low->B
            self.steering_dac.output_ab(
                ranges::coerce(SteeringSpoofHighSignal::clamp(spoof_command_high)),
                ranges::coerce(SteeringSpoofLowSignal::clamp(spoof_command_low)),
            );
        }
    }

    pub fn check_for_faults(
        &mut self,
        timer_ms: &MsTimer,
        debug_console: &mut DebugConsole,
    ) -> Option<&OsccFaultReport> {
        if !self.control_state.enabled && !self.control_state.dtcs.are_any_set() {
            // Assumes this module already went through the proper transition into a faulted
            // and disabled state, and we do not want to double-report a possible duplicate
            // fault.
            return None;
        }

        self.steering_torque.update();

        let unfiltered_diff = self.steering_torque.diff();

        if self.filtered_diff == 0 {
            self.filtered_diff = unfiltered_diff;
        }

        // TODO - revist this
        // OSCC goes back and forth with u16 and f32 types
        self.filtered_diff = self.exponential_moving_average(
            FILTER_ALPHA,
            f32::from(unfiltered_diff),
            f32::from(self.filtered_diff),
        ) as _;

        let inputs_grounded: bool = self.grounded_fault_state.check_voltage_grounded(
            &self.steering_torque,
            FAULT_HYSTERESIS,
            timer_ms,
        );

        // sensor pins tied to ground - a value of zero indicates disconnection
        if inputs_grounded {
            self.disable_control(debug_console);

            self.control_state
                .dtcs
                .set(OSCC_STEERING_DTC_INVALID_SENSOR_VAL);

            self.update_fault_report();

            writeln!(debug_console, "Bad value read from torque sensor");

            Some(&self.fault_report)
        } else if (self.filtered_diff > TORQUE_DIFFERENCE_OVERRIDE_THRESHOLD)
            && !self.control_state.operator_override
        {
            self.disable_control(debug_console);

            self.control_state
                .dtcs
                .set(OSCC_STEERING_DTC_OPERATOR_OVERRIDE);

            self.update_fault_report();

            self.control_state.operator_override = true;

            writeln!(debug_console, "Steering operator override");

            Some(&self.fault_report)
        } else {
            self.control_state.dtcs.clear_all();
            self.control_state.operator_override = false;
            None
        }
    }

    fn update_fault_report(&mut self) {
        self.fault_report.dtcs = self.control_state.dtcs;
    }

    fn exponential_moving_average(&self, alpha: f32, input: f32, average: f32) -> f32 {
        (alpha * input) + ((1.0 - alpha) * average)
    }

    pub fn supply_steering_report(&mut self) -> &OsccSteeringReport {
        self.steering_report.enabled = self.control_state.enabled;
        self.steering_report.operator_override = self.control_state.operator_override;
        self.steering_report.dtcs = self.control_state.dtcs;
        &self.steering_report
    }

    // TODO - error handling
    pub fn process_rx_frame(&mut self, can_frame: &CanFrame, debug_console: &mut DebugConsole) {
        if let CanFrame::DataFrame(ref frame) = can_frame {
            let id: u32 = frame.id().into();
            let data = frame.data();

            if (data[0] == OSCC_MAGIC_BYTE_0) && (data[1] == OSCC_MAGIC_BYTE_1) {
                if id == OSCC_STEERING_ENABLE_CAN_ID.into() {
                    self.enable_control(debug_console);
                } else if id == OSCC_STEERING_DISABLE_CAN_ID.into() {
                    self.disable_control(debug_console);
                } else if id == OSCC_STEERING_COMMAND_CAN_ID.into() {
                    self.process_steering_command(&OsccSteeringCommand::from(frame));
                } else if id == OSCC_FAULT_REPORT_CAN_ID.into() {
                    self.process_fault_report(&OsccFaultReport::from(frame), debug_console);
                }
            }
        }
    }

    fn process_fault_report(
        &mut self,
        fault_report: &OsccFaultReport,
        debug_console: &mut DebugConsole,
    ) {
        self.disable_control(debug_console);

        writeln!(
            debug_console,
            "Fault report received from: {} DTCs: {}",
            fault_report.fault_origin_id, fault_report.dtcs
        );
    }

    fn process_steering_command(&mut self, command: &OsccSteeringCommand) {
        let clamped_torque = num::clamp(
            command.torque_request * MAXIMUM_TORQUE_COMMAND,
            MINIMUM_TORQUE_COMMAND,
            MAXIMUM_TORQUE_COMMAND,
        );

        let spoof_voltage_low: f32 = num::clamp(
            steering_torque_to_volts_low(clamped_torque),
            STEERING_SPOOF_LOW_SIGNAL_VOLTAGE_MIN,
            STEERING_SPOOF_LOW_SIGNAL_VOLTAGE_MAX,
        );

        let spoof_voltage_high: f32 = num::clamp(
            steering_torque_to_volts_high(clamped_torque),
            STEERING_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN,
            STEERING_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX,
        );

        let spoof_value_low = (STEPS_PER_VOLT * spoof_voltage_low) as u16;
        let spoof_value_high = (STEPS_PER_VOLT * spoof_voltage_high) as u16;

        self.update_steering(spoof_value_high, spoof_value_low);
    }
}
