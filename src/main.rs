#![no_std]
#![no_main]
#![feature(const_fn)]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
#[cfg(feature = "panic-over-semihosting")]
extern crate cortex_m_semihosting;
extern crate nucleo_f767zi;
extern crate num;
#[cfg(feature = "panic-over-abort")]
extern crate panic_abort;
#[cfg(feature = "panic-over-semihosting")]
extern crate panic_semihosting;

mod adc_signal;
mod board;
mod can_gateway_module;
mod config;
mod dac_mcp49xx;
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

use board::{hard_fault_indicator, Board};
use brake_module::BrakeModule;
use can_gateway_module::CanGatewayModule;
use core::fmt::Write;
use nucleo_f767zi::hal::can::RxFifo;
use nucleo_f767zi::hal::prelude::*;
use nucleo_f767zi::led;
use rt::ExceptionFrame;
use steering_module::SteeringModule;
use throttle_module::ThrottleModule;

entry!(main);

fn main() -> ! {
    // once the organization is cleaned up, the entire board doesn't need to be
    // mutable let Board {mut leds, mut delay, ..} = Board::new();
    let mut board = Board::new();

    // turn on the blue LED
    board.leds[led::Color::Blue].on();

    // show startup message and reset warnings if debugging
    #[cfg(debug_assertions)]
    {
        writeln!(board.debug_console, "oxcc is running").unwrap();

        if board.reset_conditions.low_power {
            writeln!(board.debug_console, "WARNING: low-power reset detected").unwrap();
        }
        if board.reset_conditions.window_watchdog || board.reset_conditions.independent_watchdog {
            writeln!(board.debug_console, "WARNING: watchdog reset detected").unwrap();
        }
        if board.reset_conditions.software {
            writeln!(board.debug_console, "WARNING: software reset detected").unwrap();
        }
        if board.reset_conditions.por_pdr {
            writeln!(board.debug_console, "WARNING: POR/PDR reset detected").unwrap();
        }
        if board.reset_conditions.pin {
            writeln!(board.debug_console, "WARNING: PIN reset detected").unwrap();
        }
        if board.reset_conditions.bor {
            writeln!(board.debug_console, "WARNING: BOR reset detected").unwrap();
        }
    }

    let mut brake = BrakeModule::new();
    let mut throttle = ThrottleModule::new();
    let mut steering = SteeringModule::new();
    let mut can_gateway = CanGatewayModule::new();

    brake.init_devices(&mut board);
    throttle.init_devices(&mut board);
    steering.init_devices(&mut board);
    can_gateway.init_devices(&mut board);

    // send reports immediately
    brake.publish_brake_report(&mut board);
    throttle.publish_throttle_report(&mut board);
    steering.publish_steering_report(&mut board);

    loop {
        // refresh the independent watchdog
        board.wdg.refresh();

        // poll both control CAN FIFOs
        for fifo in [RxFifo::Fifo0, RxFifo::Fifo1].iter() {
            if let Ok(rx_frame) = board.control_can().receive(fifo) {
                brake.process_rx_frame(&rx_frame, &mut board);
                throttle.process_rx_frame(&rx_frame, &mut board);
                steering.process_rx_frame(&rx_frame, &mut board);
            }
        }

        brake.check_for_faults(&mut board);
        throttle.check_for_faults(&mut board);
        steering.check_for_faults(&mut board);

        // poll both OBD CAN FIFOs
        for fifo in [RxFifo::Fifo0, RxFifo::Fifo1].iter() {
            if let Ok(rx_frame) = board.obd_can().receive(fifo) {
                can_gateway.republish_obd_frame_to_control_can_bus(&rx_frame, &mut board);
            }
        }

        // TODO - just polling the publish timer for now
        // we can also drive this logic from the interrupt
        // handler if the objects are global and atomic
        if let Ok(_) = board.can_publish_timer.wait() {
            board.leds[led::Color::Green].toggle();

            brake.publish_brake_report(&mut board);
            throttle.publish_throttle_report(&mut board);
            steering.publish_steering_report(&mut board);

            //writeln!(board.debug_console, "{}", board.timer_ms.ms());
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
