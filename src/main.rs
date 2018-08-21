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

mod board;
mod can_gateway_module;
mod config;
mod dac_mcp4922;
mod dtc;
mod dual_signal;
mod fault_condition;
mod ms_timer;
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
use brake_module::BrakeModule;
use can_gateway_module::CanGatewayModule;
use core::fmt::Write;
use fault_can_protocol::FaultReportPublisher;
use nucleo_f767zi::hal::can::RxFifo;
use nucleo_f767zi::led;
use rt::ExceptionFrame;
use steering_module::SteeringModule;
use throttle_module::UnpreparedThrottleModule;

const DEBUG_WRITE_FAILURE: &str = "Failed to write to debug_console";

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
    board.leds[led::Color::Blue].on();

    // show startup message and reset warnings if debugging
    #[cfg(debug_assertions)]
    {
        writeln!(debug_console, "oxcc is running").unwrap();

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

    let mut brake = BrakeModule::new(brake_dac, brake_pins, brake_pedal_position_sensor);
    let unprepared_throttle_module =
        UnpreparedThrottleModule::new(accelerator_position_sensor, throttle_dac, throttle_pins);
    let mut steering = SteeringModule::new(torque_sensor, steering_dac, steering_pins);
    let mut can_gateway = CanGatewayModule::new(can_publish_timer, control_can, obd_can);

    brake.init_devices();
    let mut throttle = unprepared_throttle_module.prepare_module();
    steering.init_devices();

    // send reports immediately
    brake.publish_brake_report(&mut can_gateway);
    throttle.publish_throttle_report(&mut can_gateway);
    steering.publish_steering_report(&mut can_gateway);

    loop {
        // refresh the independent watchdog
        board.wdg.refresh();

        // poll both control CAN FIFOs
        for fifo in &[RxFifo::Fifo0, RxFifo::Fifo1] {
            match can_gateway.control_can().receive(fifo) {
                Ok(rx_frame) => {
                    brake.process_rx_frame(&rx_frame, &mut debug_console);
                    throttle.process_rx_frame(&rx_frame, &mut debug_console);
                    steering.process_rx_frame(&rx_frame, &mut debug_console);
                }
                Err(e) => writeln!(debug_console, "CAN receive failure: {:?}", e)
                    .expect(DEBUG_WRITE_FAILURE), // TODO - CAN receive error handling
            }
        }

        brake.check_for_faults(&timer_ms, &mut debug_console, &mut can_gateway);
        if let Some(throttle_fault) = throttle.check_for_faults(&timer_ms, &mut debug_console) {
            let _ = can_gateway.publish_fault_report(throttle_fault); // TODO - high-level publish error handling
        }
        steering.check_for_faults(&timer_ms, &mut debug_console, &mut can_gateway);

        can_gateway.republish_obd_frames_to_control_can_bus();

        // TODO - just polling the publish timer for now
        // we can also drive this logic from the interrupt
        // handler if the objects are global and atomic
        if can_gateway.wait_for_publish() {
            board.leds[led::Color::Green].toggle();

            brake.publish_brake_report(&mut can_gateway);
            throttle.publish_throttle_report(&mut can_gateway);
            steering.publish_steering_report(&mut can_gateway);
        }

        // TODO - do anything with the user button?
        if board.user_button() {
            // can only do this when we're debugging/semihosting
            #[cfg(feature = "panic-over-semihosting")]
            cortex_m::asm::bkpt();
        }
    }
}

exception!(HardFault, hard_fault);

// TODO - any safety related things we can do in these contexts (disable
// controls, LEDs, etc)?
fn hard_fault(ef: &ExceptionFrame) -> ! {
    hard_fault_indicator();
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    hard_fault_indicator();
    panic!("Unhandled exception (IRQn = {})", irqn);
}
