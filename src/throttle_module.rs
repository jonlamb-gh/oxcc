// https://github.com/jonlamb-gh/oscc/tree/devel/firmware/throttle

use board::AcceleratorPositionSensor;
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
use throttle_can_protocol::*;
use types::*;
use vehicle::*;

struct ThrottleControlState<DTCS: DtcBitfield> {
    enabled: bool,
    operator_override: bool,
    dtcs: DTCS,
}

impl<DTCS> ThrottleControlState<DTCS>
where
    DTCS: DtcBitfield,
{
    pub const fn new(dtcs: DTCS) -> Self {
        ThrottleControlState {
            enabled: false,
            operator_override: false,
            dtcs,
        }
    }
}

pub struct ThrottleModule {
    accelerator_position: DualSignal<AcceleratorPositionSensor>,
    control_state: ThrottleControlState<u8>,
    grounded_fault_state: FaultCondition,
    operator_override_state: FaultCondition,
    throttle_report: OsccThrottleReport,
    fault_report: OsccFaultReport,
    throttle_dac: ThrottleDac,
    throttle_pins: ThrottlePins,
}

pub struct UnpreparedThrottleModule {
    throttle_module: ThrottleModule,
}

impl UnpreparedThrottleModule {
    pub fn new(
        accelerator_position_sensor: AcceleratorPositionSensor,
        throttle_dac: ThrottleDac,
        throttle_pins: ThrottlePins,
    ) -> UnpreparedThrottleModule {
        UnpreparedThrottleModule {
            throttle_module: ThrottleModule {
                accelerator_position: DualSignal::new(0, 0, accelerator_position_sensor),
                control_state: ThrottleControlState::new(u8::default()),
                grounded_fault_state: FaultCondition::new(),
                operator_override_state: FaultCondition::new(),
                throttle_report: OsccThrottleReport::new(),
                fault_report: OsccFaultReport {
                    fault_origin_id: FAULT_ORIGIN_THROTTLE,
                    dtcs: 0,
                },
                throttle_dac,
                throttle_pins,
            },
        }
    }

    pub fn prepare_module(self) -> ThrottleModule {
        let mut throttle_module = self.throttle_module;
        throttle_module.throttle_pins.spoof_enable.set_low();
        throttle_module
    }
}

impl ThrottleModule {
    fn disable_control(&mut self, debug_console: &mut DebugConsole) {
        if self.control_state.enabled {
            self.accelerator_position.prevent_signal_discontinuity();

            self.throttle_dac.output_ab(
                DacOutput::clamp(self.accelerator_position.low()),
                DacOutput::clamp(self.accelerator_position.high()),
            );

            self.throttle_pins.spoof_enable.set_low();
            self.control_state.enabled = false;
            writeln!(debug_console, "Throttle control disabled");
        }
    }

    fn enable_control(&mut self, debug_console: &mut DebugConsole) {
        if !self.control_state.enabled && !self.control_state.operator_override {
            self.accelerator_position.prevent_signal_discontinuity();

            self.throttle_dac.output_ab(
                DacOutput::clamp(self.accelerator_position.low()),
                DacOutput::clamp(self.accelerator_position.high()),
            );

            self.throttle_pins.spoof_enable.set_high();
            self.control_state.enabled = true;
            writeln!(debug_console, "Throttle control enabled");
        }
    }

    fn update_throttle(&mut self, spoof_command_high: u16, spoof_command_low: u16) {
        if self.control_state.enabled {
            // TODO - revisit this, enforce high->A, low->B
            self.throttle_dac.output_ab(
                ranges::coerce(ThrottleSpoofHighSignal::clamp(spoof_command_high)),
                ranges::coerce(ThrottleSpoofHighSignal::clamp(spoof_command_low)),
            );
        }
    }

    /// Checks for any fresh (previously undetected or unhandled) faults
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

        self.accelerator_position.update();

        let accelerator_position_average = self.accelerator_position.average();

        let operator_overridden: bool = self.operator_override_state.condition_exceeded_duration(
            accelerator_position_average >= ACCELERATOR_OVERRIDE_THRESHOLD,
            FAULT_HYSTERESIS,
            timer_ms,
        );

        let inputs_grounded: bool = self.grounded_fault_state.check_voltage_grounded(
            &self.accelerator_position,
            FAULT_HYSTERESIS,
            timer_ms,
        );

        // sensor pins tied to ground - a value of zero indicates disconnection
        if inputs_grounded {
            self.disable_control(debug_console);

            self.control_state
                .dtcs
                .set(OSCC_THROTTLE_DTC_INVALID_SENSOR_VAL);

            self.update_fault_report();

            writeln!(
                debug_console,
                "Bad value read from accelerator position sensor"
            );

            Some(&self.fault_report)
        } else if operator_overridden && !self.control_state.operator_override {
            self.disable_control(debug_console);

            self.control_state
                .dtcs
                .set(OSCC_THROTTLE_DTC_OPERATOR_OVERRIDE);

            self.update_fault_report();

            self.control_state.operator_override = true;

            writeln!(debug_console, "Throttle operator override");

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

    pub fn supply_throttle_report(&mut self) -> &OsccThrottleReport {
        self.throttle_report.enabled = self.control_state.enabled;
        self.throttle_report.operator_override = self.control_state.operator_override;
        self.throttle_report.dtcs = self.control_state.dtcs;
        &self.throttle_report
    }

    // TODO - error handling
    pub fn process_rx_frame(&mut self, can_frame: &CanFrame, debug_console: &mut DebugConsole) {
        if let CanFrame::DataFrame(ref frame) = can_frame {
            let id: u32 = frame.id().into();
            let data = frame.data();

            if (data[0] == OSCC_MAGIC_BYTE_0) && (data[1] == OSCC_MAGIC_BYTE_1) {
                if id == OSCC_THROTTLE_ENABLE_CAN_ID.into() {
                    self.enable_control(debug_console);
                } else if id == OSCC_THROTTLE_DISABLE_CAN_ID.into() {
                    self.disable_control(debug_console);
                } else if id == OSCC_THROTTLE_COMMAND_CAN_ID.into() {
                    self.process_throttle_command(&OsccThrottleCommand::from(frame));
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

    fn process_throttle_command(&mut self, command: &OsccThrottleCommand) {
        let clamped_position = num::clamp(
            command.torque_request,
            MINIMUM_THROTTLE_COMMAND,
            MAXIMUM_THROTTLE_COMMAND,
        );

        let spoof_voltage_low: f32 = num::clamp(
            throttle_position_to_volts_low(clamped_position),
            THROTTLE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN,
            THROTTLE_SPOOF_LOW_SIGNAL_VOLTAGE_MAX,
        );

        let spoof_voltage_high: f32 = num::clamp(
            throttle_position_to_volts_high(clamped_position),
            THROTTLE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN,
            THROTTLE_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX,
        );

        let spoof_value_low = (STEPS_PER_VOLT * spoof_voltage_low) as u16;
        let spoof_value_high = (STEPS_PER_VOLT * spoof_voltage_high) as u16;

        self.update_throttle(spoof_value_high, spoof_value_low);
    }
}
