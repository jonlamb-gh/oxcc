#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_semihosting as sh;
extern crate nucleo_f767zi;
extern crate panic_semihosting;
#[macro_use(block)]
extern crate nb;

mod fault_condition;
mod pid;
mod throttle_module;

use core::fmt::Write;
use cortex_m::peripheral::syst::SystClkSource;
use nucleo_f767zi::hal::flash::FlashExt;
use nucleo_f767zi::hal::prelude::*;
use nucleo_f767zi::hal::serial::*;
use nucleo_f767zi::hal::stm32f7x7;
use nucleo_f767zi::led;
use nucleo_f767zi::led::Leds;
use rt::ExceptionFrame;
use sh::hio;

entry!(main);

fn main() -> ! {
    let mut stdout = hio::hstdout().unwrap();
    writeln!(stdout, "Starting").unwrap();

    let cm_peripherals = cortex_m::Peripherals::take().unwrap();
    let peripherals = stm32f7x7::Peripherals::take().unwrap();

    let mut flash = peripherals.FLASH.constrain();
    let mut rcc = peripherals.RCC.constrain();

    let gpiob = peripherals.GPIOB.split(&mut rcc.ahb);
    let gpiod = peripherals.GPIOD.split(&mut rcc.ahb);

    let mut syst = cm_peripherals.SYST;

    // TODO - fix/enable RCC bits in HAL
    //let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut leds = Leds::new(gpiob);
    for led in leds.iter_mut() {
        led.off();
    }
    leds[led::Color::Blue].on();

    // TODO - need RCC bits to get full clock speed
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(8_000_000); // 1s
    syst.enable_counter();

    /* TODO - need RCC bits for this
    let tx = gpiod.pd8.into_af7(&mut gpiod.moder, &mut gpiod.afrh);
    let rx = gpiod.pd9.into_af7(&mut gpiod.moder, &mut gpiod.afrh);
    let serial = Serial::usart3(p.USART3, (tx, rx), 9_600.bps(), clocks, &mut rcc.apb1);
    let (mut tx, mut rx) = serial.split();

    let sent = b'X';
    block!(tx.write(sent)).ok();
    */

    let mut led_state = false;
    loop {
        while !syst.has_wrapped() {}

        if led_state {
            leds[led::Color::Green].on();
        } else {
            leds[led::Color::Green].off();
        }
        led_state = !led_state;

        writeln!(stdout, "SYST wrapped").unwrap();
    }
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
