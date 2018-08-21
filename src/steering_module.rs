// https://github.com/jonlamb-gh/oscc/tree/devel/firmware/steering

use board::TorqueSensor;
use can_gateway_module::CanGatewayModule;
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
use steering_can_protocol::*;
use types::*;
use vehicle::*;

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
    steering_torque: DualSignal<TorqueSensor>,
    control_state: SteeringControlState,
    grounded_fault_state: FaultCondition,
    filtered_diff: u16,
    steering_report: OsccSteeringReport,
    fault_report: OsccFaultReport,
    steering_dac: SteeringDac,
    steering_pins: SteeringPins,
}

impl SteeringModule {
    pub fn new(
        torque_sensor: TorqueSensor,
        steering_dac: SteeringDac,
        steering_pins: SteeringPins,
    ) -> Self {
        SteeringModule {
            steering_torque: DualSignal::new(0, 0, torque_sensor),
            control_state: SteeringControlState::new(),
            grounded_fault_state: FaultCondition::new(),
            filtered_diff: 0,
            steering_report: OsccSteeringReport::new(),
            fault_report: OsccFaultReport {
                fault_origin_id: FAULT_ORIGIN_STEERING,
                dtcs: 0,
            },
            steering_dac,
            steering_pins,
        }
    }

    pub fn init_devices(&mut self) {
        self.steering_pins.spoof_enable.set_low();
    }

    pub fn disable_control(&mut self, debug_console: &mut DebugConsole) {
        if self.control_state.enabled {
            self.steering_torque.prevent_signal_discontinuity();

            self.steering_dac
                .output_ab(self.steering_torque.low(), self.steering_torque.high());

            self.steering_pins.spoof_enable.set_low();
            self.control_state.enabled = false;
            writeln!(debug_console, "Steering control disabled");
        }
    }

    pub fn enable_control(&mut self, debug_console: &mut DebugConsole) {
        if !self.control_state.enabled && !self.control_state.operator_override {
            self.steering_torque.prevent_signal_discontinuity();

            self.steering_dac
                .output_ab(self.steering_torque.low(), self.steering_torque.high());

            self.steering_pins.spoof_enable.set_high();
            self.control_state.enabled = true;
            writeln!(debug_console, "Steering control enabled");
        }
    }

    pub fn update_steering(&mut self, spoof_command_high: u16, spoof_command_low: u16) {
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
            self.steering_dac.output_ab(spoof_high, spoof_low);
        }
    }

    pub fn check_for_faults<P: FaultReportPublisher>(
        &mut self,
        timer_ms: &MsTimer,
        debug_console: &mut DebugConsole,
        fault_report_publisher: &mut P,
    ) {
        if self.control_state.enabled || self.control_state.dtcs > 0 {
            self.read_torque_sensor();

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

                if fault_report_publisher
                    .publish_fault_report(self.supply_fault_report())
                    .is_err()
                {
                    // TODO - publish error handling
                }

                writeln!(debug_console, "Bad value read from torque sensor");
            } else if (self.filtered_diff > TORQUE_DIFFERENCE_OVERRIDE_THRESHOLD)
                && !self.control_state.operator_override
            {
                // TODO - oxcc change, don't continously disable when override is already
                // handled oscc throttle module doesn't allow for continious
                // override-disables: https://github.com/jonlamb-gh/oscc/blob/master/firmware/throttle/src/throttle_control.cpp#L64
                // but brake and steering do?
                // https://github.com/jonlamb-gh/oscc/blob/master/firmware/brake/kia_soul_ev_niro/src/brake_control.cpp#L65
                // https://github.com/jonlamb-gh/oscc/blob/master/firmware/steering/src/steering_control.cpp#L84
                self.disable_control(debug_console);

                self.control_state
                    .dtcs
                    .set(OSCC_STEERING_DTC_OPERATOR_OVERRIDE);

                if fault_report_publisher
                    .publish_fault_report(self.supply_fault_report())
                    .is_err()
                {
                    // TODO - publish error handling
                }

                self.control_state.operator_override = true;

                writeln!(debug_console, "Steering operator override");
            } else {
                self.control_state.dtcs = 0;
                self.control_state.operator_override = false;
            }
        }
    }

    fn exponential_moving_average(&self, alpha: f32, input: f32, average: f32) -> f32 {
        (alpha * input) + ((1.0 - alpha) * average)
    }

    pub fn publish_steering_report(&mut self, can_gateway: &mut CanGatewayModule) {
        self.steering_report.enabled = self.control_state.enabled;
        self.steering_report.operator_override = self.control_state.operator_override;
        self.steering_report.dtcs = self.control_state.dtcs;

        self.steering_report
            .transmit(&mut can_gateway.control_can());
    }

    fn supply_fault_report(&mut self) -> &OsccFaultReport {
        self.fault_report.fault_origin_id = FAULT_ORIGIN_STEERING;
        self.fault_report.dtcs = self.control_state.dtcs;
        &self.fault_report
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

    fn read_torque_sensor(&mut self) {
        self.steering_torque.update();
    }
}
