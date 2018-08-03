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

mod debug;
mod fault_condition;
mod pid;
mod throttle_module;

use core::fmt::Write;
use debug::DebugOutputHandle;
use nucleo_f767zi::hal::delay::Delay;
use nucleo_f767zi::hal::flash::FlashExt;
use nucleo_f767zi::hal::prelude::*;
use nucleo_f767zi::hal::serial::Serial;
use nucleo_f767zi::hal::stm32f7x7;
use nucleo_f767zi::led;
use nucleo_f767zi::led::Leds;
use rt::ExceptionFrame;
use sh::hio;

entry!(main);

fn main() -> ! {
    // stdout is routed through stlink semihosting
    let mut stdout = hio::hstdout().unwrap();
    writeln!(stdout, "Starting").unwrap();

    let core_peripherals = cortex_m::Peripherals::take().unwrap();
    let peripherals = stm32f7x7::Peripherals::take().unwrap();

    let mut flash = peripherals.FLASH.constrain();
    let mut rcc = peripherals.RCC.constrain();

    let mut gpiob = peripherals.GPIOB.split(&mut rcc.ahb1);
    let mut gpiod = peripherals.GPIOD.split(&mut rcc.ahb1);

    // default clock configuration runs at 16 MHz
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    //
    // TODO - alternate clock configuration, breaks delay currently
    //let clocks = rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz()).freeze(&mut
    // flash.acr);

    writeln!(stdout, "sysclk = {} Hz", clocks.sysclk().0);
    writeln!(stdout, "pclk1 = {} Hz", clocks.pclk1().0);
    writeln!(stdout, "pclk2 = {} Hz", clocks.pclk2().0);

    let mut leds = Leds::new(gpiob);
    for led in leds.iter_mut() {
        led.off();
    }
    leds[led::Color::Blue].on();

    let mut delay = Delay::new(core_peripherals.SYST, clocks);

    let tx = gpiod.pd8.into_af7(&mut gpiod.moder, &mut gpiod.afrh);
    let rx = gpiod.pd9.into_af7(&mut gpiod.moder, &mut gpiod.afrh);

    // USART3 is routed up to the same USB port as the stlink
    // shows up as /dev/ttyACM0 for me
    let serial = Serial::usart3(
        peripherals.USART3,
        (tx, rx),
        115_200.bps(),
        clocks,
        &mut rcc.apb1,
    );
    let (mut tx, _rx) = serial.split();

    let mut debugout = DebugOutputHandle::init(&mut tx);

    let mut led_state = false;
    loop {
        if led_state {
            leds[led::Color::Green].on();
        } else {
            leds[led::Color::Green].off();
        }
        led_state = !led_state;

        writeln!(debugout, "Message on the debug console");

        // 1 second
        delay.delay_ms(1_000_u16);
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
