use core::fmt::Write;
use cortex_m;
use dac_mcp49xx::Mcp49xx;
use ms_timer::MsTimer;
use nucleo_f767zi::can::{Can1, Can2};
use nucleo_f767zi::debug_console::DebugConsole;
use nucleo_f767zi::hal::delay::Delay;
use nucleo_f767zi::hal::gpio::gpioa::PA3;
use nucleo_f767zi::hal::gpio::gpioc::{PC0, PC3};
use nucleo_f767zi::hal::gpio::gpiod::{PD10, PD11, PD12, PD13};
use nucleo_f767zi::hal::gpio::gpiof::{PF10, PF3, PF5};
use nucleo_f767zi::hal::gpio::{Floating, Input, Output, PushPull};
use nucleo_f767zi::hal::prelude::*;
use nucleo_f767zi::hal::serial::Serial;
use nucleo_f767zi::hal::stm32f7x7;
use nucleo_f767zi::hal::stm32f7x7::{ADC1, Interrupt, TIM2, NVIC, RCC};
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
type AcceleratorPositionSensorHigh = PA3<Input<Floating>>; // ADC123_IN3
type AcceleratorPositionSensorLow = PC0<Input<Floating>>; // ADC123_IN10

type SteeringSpoofEnable = PD11<Output<PushPull>>;
type TorqueSensorHigh = PC3<Input<Floating>>;
type TorqueSensorLow = PF3<Input<Floating>>;

type BrakeSpoofEnable = PD12<Output<PushPull>>;
type BrakeLightEnable = PD13<Output<PushPull>>;
type BrakePedalPositionSensorHigh = PF5<Input<Floating>>;
type BrakePedalPositionSensorLow = PF10<Input<Floating>>;

type CanPublishTimer = Timer<TIM2>;

const CAN_PUBLISH_HZ: u32 = 50;

/*
pub struct ThrottlePins {
    pub spoof_enable: ThrottleSpoofEnable,
    pub accel_pos_sensor_high: AcceleratorPositionSensorHigh,
    pub accel_pos_sensor_low: AcceleratorPositionSensorLow,
}
*/

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
    // TODO - testing
    pub accel_pos_sensor_high: AcceleratorPositionSensorHigh,
    pub accel_pos_sensor_low: AcceleratorPositionSensorLow,
}

impl Board {
    pub fn new() -> Self {
        let mut semihost_console = hio::hstdout().unwrap();
        writeln!(semihost_console, "System starting").unwrap();

        let core_peripherals = cortex_m::Peripherals::take().unwrap();
        let peripherals = stm32f7x7::Peripherals::take().unwrap();

        let mut flash = peripherals.FLASH.constrain();
        let mut rcc = peripherals.RCC.constrain();
        let mut nvic = core_peripherals.NVIC;

        let gpiob = peripherals.GPIOB.split(&mut rcc.ahb1);
        let mut gpiod = peripherals.GPIOD.split(&mut rcc.ahb1);
        let mut gpioa = peripherals.GPIOA.split(&mut rcc.ahb1);
        let mut gpioc = peripherals.GPIOC.split(&mut rcc.ahb1);

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
        let accel_pos_sensor_high = gpioa
            .pa3
            .into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr);
        let accel_pos_sensor_low = gpioc
            .pc0
            .into_floating_input(&mut gpioc.moder, &mut gpioc.pupdr);

        let usart3_tx = gpiod.pd8.into_af7(&mut gpiod.moder, &mut gpiod.afrh);
        let usart3_rx = gpiod.pd9.into_af7(&mut gpiod.moder, &mut gpiod.afrh);

        // default clock configuration runs at 16 MHz
        let clocks = rcc.cfgr.freeze(&mut flash.acr);
        //
        // TODO - alternate clock configuration, breaks delay currently
        // need to check timer impl as well with this change
        //let clocks = rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz()).freeze(&mut
        // flash.acr);

        // configure and start the ADC conversions
        start_adc(&mut nvic);

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
            accel_pos_sensor_high,
            accel_pos_sensor_low,
        }
    }
}

// TODO - this isn't working yet
fn start_adc(nvic: &mut NVIC) {
    // TODO - need to enable safe API bits in the HAL crate
    cortex_m::interrupt::free(|_cs| {
        let rcc = unsafe { &*RCC::ptr() };
        let adc1 = unsafe { &*ADC1::ptr() };

        // enable ADC123 peripheral clocks
        rcc.apb2enr.write(|w| w.adc1en().set_bit());
        rcc.apb2enr.write(|w| w.adc2en().set_bit());
        rcc.apb2enr.write(|w| w.adc3en().set_bit());

        // TODO - our device svd file seems to be missing the CCR?
        // set ADC prescaler, PCLK2 divided by 8
        //adc1.ccr.write(|w| w.adcpre().bits(0b11));

        // disable overrun interrupt
        adc1.cr1.write(|w| w.ovrie().clear_bit());

        // 12-bit resolution
        adc1.cr1.write(|w| w.res().bits(0b00));

        // enable scan mode
        adc1.cr1.write(|w| w.scan().set_bit());

        // disable analog watchdog
        adc1.cr1.write(|w| w.awden().clear_bit());
        adc1.cr1.write(|w| w.jawden().clear_bit());

        // enable end of conversion interrupt
        adc1.cr1.write(|w| w.eocie().set_bit());

        // right alignment
        adc1.cr2.write(|w| w.align().clear_bit());

        // EOC set at the end of each sequence
        adc1.cr2.write(|w| w.eocs().clear_bit());

        // TODO - update this with all AINs
        // sequence length, 2 channels
        adc1.sqr1.write(|w| w.l().bits(1));

        // TODO - ADC_SQRx channel configs IN3, IN10
        // channel conversion sequence in order
        // IN3, IN10
        adc1.sqr3.write(|w| unsafe { w.sq1().bits(3) });
        adc1.sqr3.write(|w| unsafe { w.sq2().bits(10) });

        // TODO - ADC_SMPRx - sample time 480 cycles
        adc1.smpr2.write(|w| unsafe { w.smp0().bits(0b111) });
        adc1.smpr2.write(|w| unsafe { w.smp1().bits(0b111) });

        // continuous conversion mode
        adc1.cr2.write(|w| w.cont().set_bit());

        // power on
        adc1.cr2.write(|w| w.adon().set_bit());

        // TODO - calibration?

        // start conversion of regular channels
        adc1.cr2.write(|w| w.swstart().set_bit());

        // enable ADC interrupt
        nvic.enable(Interrupt::ADC);

        // TODO - print out status registers
    });
}
