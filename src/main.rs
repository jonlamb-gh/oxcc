#![no_std]
#![no_main]
#![feature(const_fn)]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
#[cfg(feature = "panic-over-semihosting")]
extern crate cortex_m_semihosting;
extern crate embedded_hal;
extern crate nucleo_f767zi;
extern crate num;
#[cfg(feature = "panic-over-abort")]
extern crate panic_abort;
#[cfg(feature = "panic-over-semihosting")]
extern crate panic_semihosting;
#[macro_use]
extern crate typenum;

mod board;
mod can_gateway_module;
mod config;
mod dac_mcp4922;
mod dtc;
mod dual_signal;
mod fault_condition;
mod ms_timer;
mod oxcc_error;
mod ranges;
mod steering_module;
mod throttle_module;
mod types;

#[path = "can_protocols/brake_can_protocol.rs"]
mod brake_can_protocol;
#[path = "can_protocols/fault_can_protocol.rs"]
mod fault_can_protocol;
#[path = "can_protocols/oscc_magic_byte.rs"]
mod oscc_magic_byte;
#[path = "can_protocols/steering_can_protocol.rs"]
mod steering_can_protocol;
#[path = "can_protocols/throttle_can_protocol.rs"]
mod throttle_can_protocol;

#[cfg(feature = "kia-niro")]
#[path = "vehicles/kial_niro.rs"]
mod kial_niro;
#[cfg(feature = "kia-soul-ev")]
#[path = "vehicles/kial_soul_ev.rs"]
mod kial_soul_ev;
#[cfg(feature = "kia-soul-petrol")]
#[path = "vehicles/kial_soul_petrol.rs"]
mod kial_soul_petrol;
mod vehicle;

#[cfg(any(feature = "kia-soul-ev", feature = "kia-niro"))]
#[path = "brake/kia_soul_ev_niro/brake_module.rs"]
mod brake_module;
#[cfg(feature = "kia-soul-petrol")]
#[path = "brake/kia_soul_petrol/brake_module.rs"]
mod brake_module;

use board::{hard_fault_indicator, FullBoard};
use brake_can_protocol::BrakeReportPublisher;
use brake_module::{BrakeModule, UnpreparedBrakeModule};
use can_gateway_module::CanGatewayModule;
use core::fmt::Write;
use fault_can_protocol::FaultReportPublisher;
use ms_timer::MsTimer;
use nucleo_f767zi::debug_console::DebugConsole;
use nucleo_f767zi::hal::can::CanError;
use nucleo_f767zi::hal::can::RxFifo;
use nucleo_f767zi::led::{Color, Leds};
use oxcc_error::OxccError;
use rt::ExceptionFrame;
use steering_can_protocol::SteeringReportPublisher;
use steering_module::{SteeringModule, UnpreparedSteeringModule};
use throttle_can_protocol::ThrottleReportPublisher;
use throttle_module::{ThrottleModule, UnpreparedThrottleModule};

const DEBUG_WRITE_FAILURE: &str = "Failed to write to debug_console";

struct ControlModules {
    pub brake: BrakeModule,
    pub throttle: ThrottleModule,
    pub steering: SteeringModule,
}

entry!(main);

fn main() -> ! {
    // once the organization is cleaned up, the entire board doesn't need to be
    // mutable let Board {mut leds, mut delay, ..} = Board::new();
    let (
        mut board,
        brake_dac,
        brake_pins,
        brake_pedal_position_sensor,
        accelerator_position_sensor,
        throttle_dac,
        throttle_pins,
        torque_sensor,
        steering_dac,
        steering_pins,
        timer_ms,
        mut debug_console,
        can_publish_timer,
        control_can,
        obd_can,
    ) = FullBoard::new().split_components();

    // turn on the blue LED
    board.leds[Color::Blue].on();

    // show startup message and reset warnings if debugging
    #[cfg(debug_assertions)]
    {
        writeln!(debug_console, "OxCC is running").unwrap();

        // TODO - some of these are worthy of disabling controls?
        if board.reset_conditions.low_power {
            writeln!(debug_console, "WARNING: low-power reset detected")
                .expect(DEBUG_WRITE_FAILURE);
        }
        if board.reset_conditions.window_watchdog || board.reset_conditions.independent_watchdog {
            writeln!(debug_console, "WARNING: watchdog reset detected").expect(DEBUG_WRITE_FAILURE);
        }
        if board.reset_conditions.software {
            writeln!(debug_console, "WARNING: software reset detected").expect(DEBUG_WRITE_FAILURE);
        }
        if board.reset_conditions.por_pdr {
            writeln!(debug_console, "WARNING: POR/PDR reset detected").expect(DEBUG_WRITE_FAILURE);
        }
        if board.reset_conditions.pin {
            writeln!(debug_console, "WARNING: PIN reset detected").expect(DEBUG_WRITE_FAILURE);
        }
        if board.reset_conditions.bor {
            writeln!(debug_console, "WARNING: BOR reset detected").expect(DEBUG_WRITE_FAILURE);
        }
    }

    let unprepared_brake_module =
        UnpreparedBrakeModule::new(brake_dac, brake_pins, brake_pedal_position_sensor);
    let unprepared_throttle_module =
        UnpreparedThrottleModule::new(accelerator_position_sensor, throttle_dac, throttle_pins);
    let unprepared_steering_module =
        UnpreparedSteeringModule::new(torque_sensor, steering_dac, steering_pins);
    let mut can_gateway = CanGatewayModule::new(can_publish_timer, control_can, obd_can);

    let mut modules = ControlModules {
        brake: unprepared_brake_module.prepare_module(),
        throttle: unprepared_throttle_module.prepare_module(),
        steering: unprepared_steering_module.prepare_module(),
    };

    // send reports immediately
    if let Err(e) = publish_reports(&mut modules, &mut can_gateway) {
        handle_error(
            e,
            &mut modules,
            &mut can_gateway,
            &mut debug_console,
            &mut board.leds,
        );
    }

    loop {
        // refresh the independent watchdog
        board.wdg.refresh();

        // check the control CAN FIFOs for any frames to be processed
        if let Err(e) =
            process_control_can_frames(&mut modules, &mut can_gateway, &mut debug_console)
        {
            handle_error(
                e,
                &mut modules,
                &mut can_gateway,
                &mut debug_console,
                &mut board.leds,
            );
        }

        // check modules for fault conditions, sending reports as needed
        // NOTE
        // ignoring transmit timeouts until a proper error handling strategy is
        // implemented
        if let Err(e) = check_for_faults(
            &mut modules,
            &mut can_gateway,
            &timer_ms,
            &mut debug_console,
        ) {
            if e != OxccError::Can(CanError::Timeout) {
                handle_error(
                    e,
                    &mut modules,
                    &mut can_gateway,
                    &mut debug_console,
                    &mut board.leds,
                );
            }
        }

        // republish OBD frames to control CAN bus
        if let Err(e) = can_gateway.republish_obd_frames_to_control_can_bus() {
            handle_error(
                e,
                &mut modules,
                &mut can_gateway,
                &mut debug_console,
                &mut board.leds,
            );
        }

        // periodically publish all report frames
        if can_gateway.wait_for_publish() {
            board.leds[Color::Green].toggle();

            if let Err(e) = publish_reports(&mut modules, &mut can_gateway) {
                handle_error(
                    e,
                    &mut modules,
                    &mut can_gateway,
                    &mut debug_console,
                    &mut board.leds,
                );
            }
        }

        // TODO - do anything with the user button?
        if board.user_button() {
            // can only do this when we're debugging/semihosting
            #[cfg(feature = "panic-over-semihosting")]
            cortex_m::asm::bkpt();
        }
    }
}

fn process_control_can_frames(
    modules: &mut ControlModules,
    can_gateway: &mut CanGatewayModule,
    debug_console: &mut DebugConsole,
) -> Result<(), OxccError> {
    // poll both control CAN FIFOs
    for fifo in &[RxFifo::Fifo0, RxFifo::Fifo1] {
        match can_gateway.control_can().receive(fifo) {
            Ok(rx_frame) => {
                modules.brake.process_rx_frame(&rx_frame, debug_console)?;
                modules
                    .throttle
                    .process_rx_frame(&rx_frame, debug_console)?;
                modules
                    .steering
                    .process_rx_frame(&rx_frame, debug_console)?;
            }
            Err(e) => {
                // report all but BufferExhausted (no data)
                if e != CanError::BufferExhausted {
                    return Err(OxccError::from(e));
                }
            }
        }
    }

    Ok(())
}

fn check_for_faults(
    modules: &mut ControlModules,
    can_gateway: &mut CanGatewayModule,
    timer_ms: &MsTimer,
    debug_console: &mut DebugConsole,
) -> Result<(), OxccError> {
    let maybe_fault = modules.brake.check_for_faults(timer_ms, debug_console)?;
    if let Some(brake_fault) = maybe_fault {
        can_gateway.publish_fault_report(brake_fault)?;
    }

    let maybe_fault = modules.throttle.check_for_faults(timer_ms, debug_console)?;
    if let Some(throttle_fault) = maybe_fault {
        can_gateway.publish_fault_report(throttle_fault)?;
    }

    let maybe_fault = modules.steering.check_for_faults(timer_ms, debug_console)?;
    if let Some(steering_fault) = maybe_fault {
        can_gateway.publish_fault_report(steering_fault)?;
    }

    Ok(())
}

// NOTE
// ignoring transmit timeouts until a proper error handling strategy is
// implemented
fn publish_reports(
    modules: &mut ControlModules,
    can_gateway: &mut CanGatewayModule,
) -> Result<(), OxccError> {
    // attempt to publish them all, only report the last to fail
    let mut result = Ok(());

    // it is typically to get timeout errors if the CAN bus is not active or
    // there are no other nodes connected to it
    if let Err(e) = can_gateway.publish_brake_report(modules.brake.supply_brake_report()) {
        if e != CanError::Timeout {
            result = Err(OxccError::from(e));
        }
    }

    if let Err(e) = can_gateway.publish_throttle_report(modules.throttle.supply_throttle_report()) {
        if e != CanError::Timeout {
            result = Err(OxccError::from(e));
        }
    }

    if let Err(e) = can_gateway.publish_steering_report(modules.steering.supply_steering_report()) {
        if e != CanError::Timeout {
            result = Err(OxccError::from(e));
        }
    }

    result
}

// TODO - this is just an example for now
fn handle_error(
    error: OxccError,
    modules: &mut ControlModules,
    can_gateway: &mut CanGatewayModule,
    debug_console: &mut DebugConsole,
    leds: &mut Leds,
) {
    leds[Color::Red].on();

    writeln!(debug_console, "ERROR: {:#?}", error).expect(DEBUG_WRITE_FAILURE);

    // disable all controls
    let _ = modules.throttle.disable_control(debug_console);
    let _ = modules.steering.disable_control(debug_console);
    let _ = modules.brake.disable_control(debug_console);

    // publish reports
    let _ = publish_reports(modules, can_gateway);
}

exception!(HardFault, hard_fault);

// TODO - any safety related things we can do in these contexts?
// Might be worth implementing a panic handler here as well
// For example:
// - disable controls
// - indication LED
fn hard_fault(ef: &ExceptionFrame) -> ! {
    hard_fault_indicator();
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    hard_fault_indicator();
    panic!("Unhandled exception (IRQn = {})", irqn);
}
