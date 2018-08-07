#![no_std]
#![no_main]
#![feature(const_fn)]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_semihosting as sh;
#[macro_use]
extern crate stm32f7;
extern crate nucleo_f767zi;
extern crate num;
extern crate panic_semihosting;

mod board;
mod dac_mcp49xx;
mod dtc;
mod dual_signal;
mod fault_condition;
mod ms_timer;
mod pid;
mod steering_module;
mod throttle_module;

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
use nucleo_f767zi::hal::prelude::*;
use nucleo_f767zi::hal::stm32f7x7;
use nucleo_f767zi::led;
use rt::ExceptionFrame;
use steering_module::SteeringModule;
use throttle_module::ThrottleModule;

// Interrupt safe access
// Requires const fn's
//use core::cell::RefCell;
//use cortex_m::interrupt::Mutex;
//static THROTTLE_MODULE: Mutex<RefCell<throttle_module::ThrottleModule>> =
//    Mutex::new(RefCell::new(throttle_module::ThrottleModule::new()));

entry!(main);

fn main() -> ! {
    // once the organization is cleaned up, the entire board doesn't need to be
    // mutable let Board {mut leds, mut delay, ..} = Board::new();
    let mut board = Board::new();

    // TODO - just for testing
    board.leds[led::Color::Blue].on();

    let mut throttle = ThrottleModule::new();
    let mut steering = SteeringModule::new();

    throttle.init_devices(&mut board);
    steering.init_devices(&mut board);

    // TODO - impl for gpio::ToggleableOutputPin in BSP crate to get toggle()
    let mut led_state = false;
    loop {
        /*
         * TODO - could fill up a global buffer of sorts and sample them here
         * instead of making the objects global/atomic
         * throttle.adc_input(high, low);
         */
        if false {
            throttle.adc_input(0, 0);
            steering.adc_input(0, 0);
        }

        throttle.check_for_incoming_message(&mut board);
        steering.check_for_incoming_message(&mut board);

        throttle.check_for_faults(&mut board);
        steering.check_for_faults(&mut board);

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

            throttle.publish_throttle_report(&mut board);
            steering.publish_steering_report(&mut board);
        }
    }
}

// ADC1 global interrupt
interrupt!(ADC, adc_isr);

// TODO might have to use unsafe style like here in RCC
// https://github.com/jonlamb-gh/stm32f767-hal/blob/devel/src/rcc.rs#L262
fn adc_isr() {
    /*
    cortex_m::interrupt::free(|cs| {
        let p = stm32f7x7::Peripherals::take();
        THROTTLE_MODULE.adc_input(...);
    });
    */
}

exception!(HardFault, hard_fault);

// TODO - any safety related things we can do in these contexts (disable
// controls, LEDs, etc)?
fn hard_fault(ef: &ExceptionFrame) -> ! {
    cortex_m::interrupt::free(|_cs| unsafe {
        let peripherals = stm32f7x7::Peripherals::steal();
        let mut rcc = peripherals.RCC.constrain();
        let gpiob = peripherals.GPIOB.split(&mut rcc.ahb1);

        let mut leds = led::Leds::new(gpiob);
        leds[led::Color::Red].on();
    });

    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
