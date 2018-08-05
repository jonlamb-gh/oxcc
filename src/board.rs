// TODO - copy defs from OSCC

use core::fmt::Write;
use cortex_m;
use nucleo_f767zi::debug_console::DebugConsole;
use nucleo_f767zi::hal::delay::Delay;
use nucleo_f767zi::hal::flash::FlashExt;
use nucleo_f767zi::hal::gpio::gpiod::PD10;
use nucleo_f767zi::hal::gpio::{Output, PushPull};
use nucleo_f767zi::hal::prelude::*;
use nucleo_f767zi::hal::serial::Serial;
use nucleo_f767zi::hal::stm32f7x7;
use nucleo_f767zi::led::Leds;
use sh::hio;

// feature to pick how to route up debug_println/println?
// or
// - println! -> Serial3 console (currently debug_console)
// - debug_println! -> ITM/semihosting link

type ThrottleSpoofEnable = PD10<Output<PushPull>>;

pub struct Board {
    pub semihost_console: hio::HStdout,
    pub debug_console: DebugConsole,
    pub leds: Leds,
    pub delay: Delay,
    pub throttle_spoof_enable: ThrottleSpoofEnable,
}

impl Board {
    pub fn new() -> Self {
        let mut semihost_console = hio::hstdout().unwrap();
        writeln!(semihost_console, "System starting").unwrap();

        let core_peripherals = cortex_m::Peripherals::take().unwrap();
        let peripherals = stm32f7x7::Peripherals::take().unwrap();

        let mut flash = peripherals.FLASH.constrain();
        let mut rcc = peripherals.RCC.constrain();

        let gpiob = peripherals.GPIOB.split(&mut rcc.ahb1);
        let mut gpiod = peripherals.GPIOD.split(&mut rcc.ahb1);

        // TODO - put pin defs in board.rs, what else can be typed in BSP crate?
        // pins container for each module?
        let throttle_spoof_enable = gpiod
            .pd10
            .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);

        let usart3_tx = gpiod.pd8.into_af7(&mut gpiod.moder, &mut gpiod.afrh);
        let usart3_rx = gpiod.pd9.into_af7(&mut gpiod.moder, &mut gpiod.afrh);

        // default clock configuration runs at 16 MHz
        let clocks = rcc.cfgr.freeze(&mut flash.acr);
        //
        // TODO - alternate clock configuration, breaks delay currently
        //let clocks = rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz()).freeze(&mut
        // flash.acr);

        writeln!(semihost_console, "sysclk = {} Hz", clocks.sysclk().0);
        writeln!(semihost_console, "pclk1 = {} Hz", clocks.pclk1().0);
        writeln!(semihost_console, "pclk2 = {} Hz", clocks.pclk2().0);

        let mut leds = Leds::new(gpiob);
        for led in leds.iter_mut() {
            led.off();
        }

        let mut delay = Delay::new(core_peripherals.SYST, clocks);

        // USART3 is routed up to the same USB port as the stlink
        // shows up as /dev/ttyACM0 for me
        let serial = Serial::usart3(
            peripherals.USART3,
            (usart3_tx, usart3_rx),
            115_200.bps(),
            clocks,
            &mut rcc.apb1,
        );

        let mut debug_console = DebugConsole::new(serial);

        Board {
            semihost_console,
            debug_console,
            leds,
            delay,
            throttle_spoof_enable,
        }
    }
}
