// https://github.com/jonlamb-gh/oscc/tree/devel/firmware/brake/kia_soul_ev_niro

use super::types::*;
use board::BrakePedalPositionSensor;
use brake_can_protocol::*;
use core::fmt::Write;
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
use vehicle::*;

// TODO - use some form of println! logging that prefixes with a module name?

struct BrakeControlState<DTCS: DtcBitfield> {
    enabled: bool,
    operator_override: bool,
    dtcs: DTCS,
}

impl<DTCS> BrakeControlState<DTCS>
where
    DTCS: DtcBitfield,
{
    pub const fn new(dtcs: DTCS) -> Self {
        BrakeControlState {
            enabled: false,
            operator_override: false,
            dtcs,
        }
    }
}

pub struct BrakeModule {
    brake_pedal_position: DualSignal<BrakePedalPositionSensor>,
    control_state: BrakeControlState<u8>,
    grounded_fault_state: FaultCondition,
    operator_override_state: FaultCondition,
    brake_report: OsccBrakeReport,
    fault_report: OsccFaultReport,
    brake_dac: BrakeDac,
    brake_pins: BrakePins,
}

pub struct UnpreparedBrakeModule {
    brake_module: BrakeModule,
}

impl UnpreparedBrakeModule {
    pub fn new(
        brake_dac: BrakeDac,
        brake_pins: BrakePins,
        brake_pedal_position_sensor: BrakePedalPositionSensor,
    ) -> Self {
        UnpreparedBrakeModule {
            brake_module: BrakeModule {
                brake_pedal_position: DualSignal::new(0, 0, brake_pedal_position_sensor),
                control_state: BrakeControlState::new(u8::default()),
                grounded_fault_state: FaultCondition::new(),
                operator_override_state: FaultCondition::new(),
                brake_report: OsccBrakeReport::new(),
                fault_report: OsccFaultReport {
                    fault_origin_id: FAULT_ORIGIN_BRAKE,
                    dtcs: 0,
                },
                brake_dac,
                brake_pins,
            },
        }
    }

    pub fn prepare_module(self) -> BrakeModule {
        let mut brake_module = self.brake_module;
        brake_module.brake_pins.spoof_enable.set_low();
        brake_module.brake_pins.brake_light_enable.set_low();
        brake_module
    }
}

impl BrakeModule {
    fn disable_control(&mut self, debug_console: &mut DebugConsole) {
        if self.control_state.enabled {
            self.brake_pedal_position.prevent_signal_discontinuity();

            self.brake_dac.output_ab(
                self.brake_pedal_position.low(),
                self.brake_pedal_position.high(),
            );

            self.brake_pins.spoof_enable.set_low();
            self.brake_pins.brake_light_enable.set_low();
            self.control_state.enabled = false;
            writeln!(debug_console, "Brake control disabled");
        }
    }

    fn enable_control(&mut self, debug_console: &mut DebugConsole) {
        if !self.control_state.enabled && !self.control_state.operator_override {
            self.brake_pedal_position.prevent_signal_discontinuity();

            self.brake_dac.output_ab(
                self.brake_pedal_position.low(),
                self.brake_pedal_position.high(),
            );

            self.brake_pins.spoof_enable.set_high();
            self.control_state.enabled = true;
            writeln!(debug_console, "Brake control enabled");
        }
    }

    fn update_brake(&mut self, spoof_command_high: u16, spoof_command_low: u16) {
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
                self.brake_pins.brake_light_enable.set_high();
            } else {
                self.brake_pins.brake_light_enable.set_low();
            }

            // TODO - revisit this, enforce high->A, low->B
            self.brake_dac.output_ab(spoof_high, spoof_low);
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

        self.brake_pedal_position.update();

        let brake_pedal_position_average = self.brake_pedal_position.average();

        let operator_overridden: bool = self.operator_override_state.condition_exceeded_duration(
            brake_pedal_position_average >= BRAKE_PEDAL_OVERRIDE_THRESHOLD.into(),
            FAULT_HYSTERESIS,
            timer_ms,
        );

        let inputs_grounded: bool = self.grounded_fault_state.check_voltage_grounded(
            &self.brake_pedal_position,
            FAULT_HYSTERESIS,
            timer_ms,
        );

        // sensor pins tied to ground - a value of zero indicates disconnection
        if inputs_grounded {
            self.disable_control(debug_console);

            self.control_state
                .dtcs
                .set(OSCC_BRAKE_DTC_INVALID_SENSOR_VAL);

            self.update_fault_report();

            writeln!(
                debug_console,
                "Bad value read from brake pedal position sensor"
            );

            Some(&self.fault_report)
        } else if operator_overridden && !self.control_state.operator_override {
            // TODO - oxcc change, don't continously disable when override is already
            // handled oscc throttle module doesn't allow for continious
            // override-disables: https://github.com/jonlamb-gh/oscc/blob/master/firmware/throttle/src/throttle_control.cpp#L64
            // but brake and steering do?
            // https://github.com/jonlamb-gh/oscc/blob/master/firmware/brake/kia_soul_ev_niro/src/brake_control.cpp#L65
            // https://github.com/jonlamb-gh/oscc/blob/master/firmware/steering/src/steering_control.cpp#L84
            self.disable_control(debug_console);

            self.control_state
                .dtcs
                .set(OSCC_BRAKE_DTC_OPERATOR_OVERRIDE);

            self.update_fault_report();

            self.control_state.operator_override = true;

            writeln!(debug_console, "Brake operator override");

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

    pub fn supply_brake_report(&mut self) -> &OsccBrakeReport {
        self.brake_report.enabled = self.control_state.enabled;
        self.brake_report.operator_override = self.control_state.operator_override;
        self.brake_report.dtcs = self.control_state.dtcs;
        &self.brake_report
    }

    // TODO - error handling
    pub fn process_rx_frame(&mut self, can_frame: &CanFrame, debug_console: &mut DebugConsole) {
        if let CanFrame::DataFrame(ref frame) = can_frame {
            let id: u32 = frame.id().into();
            let data = frame.data();

            if (data[0] == OSCC_MAGIC_BYTE_0) && (data[1] == OSCC_MAGIC_BYTE_1) {
                if id == OSCC_BRAKE_ENABLE_CAN_ID.into() {
                    self.enable_control(debug_console);
                } else if id == OSCC_BRAKE_DISABLE_CAN_ID.into() {
                    self.disable_control(debug_console);
                } else if id == OSCC_BRAKE_COMMAND_CAN_ID.into() {
                    self.process_brake_command(&OsccBrakeCommand::from(frame));
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

    fn process_brake_command(&mut self, command: &OsccBrakeCommand) {
        let clamped_position = num::clamp(
            command.pedal_command,
            MINIMUM_BRAKE_COMMAND,
            MAXIMUM_BRAKE_COMMAND,
        );

        let spoof_voltage_low: f32 = num::clamp(
            brake_position_to_volts_low(clamped_position),
            BRAKE_SPOOF_LOW_SIGNAL_VOLTAGE_MIN,
            BRAKE_SPOOF_LOW_SIGNAL_VOLTAGE_MAX,
        );

        let spoof_voltage_high: f32 = num::clamp(
            brake_position_to_volts_high(clamped_position),
            BRAKE_SPOOF_HIGH_SIGNAL_VOLTAGE_MIN,
            BRAKE_SPOOF_HIGH_SIGNAL_VOLTAGE_MAX,
        );

        let spoof_value_low = (STEPS_PER_VOLT * spoof_voltage_low) as u16;
        let spoof_value_high = (STEPS_PER_VOLT * spoof_voltage_high) as u16;

        self.update_brake(spoof_value_high, spoof_value_low);
    }
}

trait HighLowReader {
    fn read_high(&self) -> u16;
    fn read_low(&self) -> u16;
}
