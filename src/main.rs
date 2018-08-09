#![no_std]
#![no_main]
#![feature(const_fn)]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_semihosting as sh;
extern crate nucleo_f767zi;
extern crate num;
extern crate panic_semihosting;

mod adc_signal;
mod board;
mod can_gateway_module;
mod dac_mcp49xx;
mod dtc;
mod dual_signal;
mod fault_condition;
mod ms_timer;
mod steering_module;
mod throttle_module;
mod types;

// TODO - feature gate
#[path = "brake/kial_soul_ev_niro/brake_module.rs"]
mod brake_module;

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

// TODO - feature gate this as vehicle
#[path = "vehicles/kial_soul_ev.rs"]
mod kial_soul_ev;

use board::Board;
use brake_module::BrakeModule;
use can_gateway_module::CanGatewayModule;
use core::fmt::Write;
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

    // TODO - just for testing
    writeln!(board.debug_console, "oxcc is running");
    board.leds[led::Color::Blue].on();

    let mut brake = BrakeModule::new();
    let mut throttle = ThrottleModule::new();
    let mut steering = SteeringModule::new();
    let mut can_gateway = CanGatewayModule::new();

    brake.init_devices(&mut board);
    throttle.init_devices(&mut board);
    steering.init_devices(&mut board);
    can_gateway.init_devices(&mut board);

    // TODO - impl for gpio::ToggleableOutputPin in BSP crate to get toggle()
    let mut led_state = false;
    loop {
        brake.check_for_incoming_message(&mut board);
        throttle.check_for_incoming_message(&mut board);
        steering.check_for_incoming_message(&mut board);

        brake.check_for_faults(&mut board);
        throttle.check_for_faults(&mut board);
        steering.check_for_faults(&mut board);

        can_gateway.republish_obd_frames_to_control_can_bus(&mut board);

        // TODO - just polling the publish timer for now
        // we can also drive this logic from the interrupt
        // handler if the objects are global and atomic
        if let Ok(_) = board.can_publish_timer.wait() {
            if led_state {
                board.leds[led::Color::Green].on();
            } else {
                board.leds[led::Color::Green].off();
            }
            led_state = !led_state;

            brake.publish_brake_report(&mut board);
            throttle.publish_throttle_report(&mut board);
            steering.publish_steering_report(&mut board);
        }
    }
}

exception!(HardFault, hard_fault);

// TODO - any safety related things we can do in these contexts (disable
// controls, LEDs, etc)?
fn hard_fault(ef: &ExceptionFrame) -> ! {
    /* steal() is not correct
    cortex_m::interrupt::free(|_cs| unsafe {
        let peripherals = stm32f7x7::Peripherals::steal();
        let mut rcc = peripherals.RCC.constrain();
        let gpiob = peripherals.GPIOB.split(&mut rcc.ahb1);

        let mut leds = led::Leds::new(gpiob);
        leds[led::Color::Red].on();
    });
    */

    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    /* steal() is not correct
    cortex_m::interrupt::free(|_cs| unsafe {
        let peripherals = stm32f7x7::Peripherals::steal();
        let mut rcc = peripherals.RCC.constrain();
        let gpiob = peripherals.GPIOB.split(&mut rcc.ahb1);

        let mut leds = led::Leds::new(gpiob);
        leds[led::Color::Red].on();
    });
    */

    panic!("Unhandled exception (IRQn = {})", irqn);
}
