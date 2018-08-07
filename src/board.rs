use core::fmt::Write;
use cortex_m;
use dac_mcp49xx::Mcp49xx;
use ms_timer::MsTimer;
use nucleo_f767zi::can::{Can1, Can2};
use nucleo_f767zi::debug_console::DebugConsole;
use nucleo_f767zi::hal::delay::Delay;
use nucleo_f767zi::hal::gpio::gpiod::{PD10, PD11, PD12, PD13};
use nucleo_f767zi::hal::gpio::{Output, PushPull};
use nucleo_f767zi::hal::prelude::*;
use nucleo_f767zi::hal::serial::Serial;
use nucleo_f767zi::hal::stm32f7x7;
use nucleo_f767zi::hal::stm32f7x7::TIM2;
use nucleo_f767zi::hal::timer::Timer;
use nucleo_f767zi::led::Leds;
use sh::hio;

// feature to pick how to route up debug_println/println?
// or
// - println! -> Serial3 console (currently debug_console)
// - debug_println! -> ITM/semihosting link

pub type ControlCan = Can1;
pub type ObdCan = Can2;

type ThrottleSpoofEnable = PD10<Output<PushPull>>;
//type AcceleratorPositionSensorHigh
//type AcceleratorPositionSensorLow
// PIN_DAC_CHIP_SELECT, etc

type SteeringSpoofEnable = PD11<Output<PushPull>>;

type BrakeSpoofEnable = PD12<Output<PushPull>>;
type BrakeLightEnable = PD13<Output<PushPull>>;

type CanPublishTimer = Timer<TIM2>;

const CAN_PUBLISH_HZ: u32 = 50;

pub struct Board {
    pub semihost_console: hio::HStdout,
    pub debug_console: DebugConsole,
    pub leds: Leds,
    pub delay: Delay,
    pub timer_ms: MsTimer,
    pub can_publish_timer: CanPublishTimer,
    pub dac: Mcp49xx,
    pub control_can: ControlCan,
    pub obd_can: ObdCan,
    pub throttle_spoof_enable: ThrottleSpoofEnable,
    pub steering_spoof_enable: SteeringSpoofEnable,
    pub brake_spoof_enable: BrakeSpoofEnable,
    pub brake_light_enable: BrakeLightEnable,
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
        let steering_spoof_enable = gpiod
            .pd11
            .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);
        let brake_spoof_enable = gpiod
            .pd12
            .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);
        let brake_light_enable = gpiod
            .pd13
            .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);

        let usart3_tx = gpiod.pd8.into_af7(&mut gpiod.moder, &mut gpiod.afrh);
        let usart3_rx = gpiod.pd9.into_af7(&mut gpiod.moder, &mut gpiod.afrh);

        // default clock configuration runs at 16 MHz
        let clocks = rcc.cfgr.freeze(&mut flash.acr);
        //
        // TODO - alternate clock configuration, breaks delay currently
        //let clocks = rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz()).freeze(&mut
        // flash.acr);

        // TODO - use the safe APIs once this block solidifies
        unsafe {
            // TODO - move this constant into BSP crate?
            // unlock registers to enable DWT cycle counter for MsTimer
            core_peripherals.DWT.lar.write(0xC5ACCE55);
        }

        writeln!(semihost_console, "sysclk = {} Hz", clocks.sysclk().0);
        writeln!(semihost_console, "pclk1 = {} Hz", clocks.pclk1().0);
        writeln!(semihost_console, "pclk2 = {} Hz", clocks.pclk2().0);

        let mut leds = Leds::new(gpiob);
        for led in leds.iter_mut() {
            led.off();
        }

        // USART3 is routed up to the same USB port as the stlink
        // shows up as /dev/ttyACM0 for me
        let serial = Serial::usart3(
            peripherals.USART3,
            (usart3_tx, usart3_rx),
            115_200.bps(),
            clocks,
            &mut rcc.apb1,
        );

        Board {
            semihost_console,
            debug_console: DebugConsole::new(serial),
            leds,
            delay: Delay::new(core_peripherals.SYST, clocks),
            timer_ms: MsTimer::new(core_peripherals.DWT, clocks),
            can_publish_timer: CanPublishTimer::tim2(
                peripherals.TIM2,
                CAN_PUBLISH_HZ.hz(),
                clocks,
                &mut rcc.apb1,
            ),
            dac: Mcp49xx::new(),
            control_can: Can1::new(),
            obd_can: Can2::new(),
            throttle_spoof_enable,
            steering_spoof_enable,
            brake_spoof_enable,
            brake_light_enable,
        }
    }
}
