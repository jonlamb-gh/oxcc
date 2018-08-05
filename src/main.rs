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
extern crate panic_semihosting;

mod board;
mod fault_condition;
mod pid;
mod throttle_module;

use board::Board;
use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::interrupt::Mutex;
use nucleo_f767zi::hal::delay::Delay;
use nucleo_f767zi::hal::prelude::*;
use nucleo_f767zi::hal::stm32f7x7;
use nucleo_f767zi::led;
use rt::ExceptionFrame;
use throttle_module::ThrottleModule;

// Interrupt safe access
/*
static THROTTLE_MODULE: Mutex<RefCell<throttle_module::ThrottleModule>> =
    Mutex::new(RefCell::new(throttle_module::ThrottleModule::new()));
*/

entry!(main);

fn main() -> ! {
    let mut board = Board::new();

    board.leds[led::Color::Blue].on();

    let throttle = ThrottleModule::new();

    let mut led_state = false;
    loop {
        if led_state {
            board.leds[led::Color::Green].on();
        } else {
            board.leds[led::Color::Green].off();
        }
        led_state = !led_state;

        writeln!(board.debug_console, "Message on the debug console");

        // 1 second
        board.delay.delay_ms(1_000_u16);
    }
}

// ADC1 global interrupt
interrupt!(ADC, adc_isr);

// TODO might have to use unsafe style like here in RCC
// https://github.com/jonlamb-gh/stm32f767-hal/blob/devel/src/rcc.rs#L262
fn adc_isr() {
    cortex_m::interrupt::free(|cs| {
        //let p = stm32f7x7::Peripherals::take();
        //THROTTLE_MODULE.adc_input(...);
    });
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
