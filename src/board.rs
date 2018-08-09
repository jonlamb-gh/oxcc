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
use nucleo_f767zi::hal::stm32f7x7::{ADC1, Interrupt, TIM2, C_ADC, NVIC, RCC};
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
        writeln!(semihost_console, "System starting");

        let mut core_peripherals = cortex_m::Peripherals::take().unwrap();
        let peripherals = stm32f7x7::Peripherals::take().unwrap();

        core_peripherals.SCB.enable_icache();
        core_peripherals
            .SCB
            .enable_dcache(&mut core_peripherals.CPUID);

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

// TODO - this will be moved into the BSP/HAL crate areas once it's developed
// TODO - need to manage bits like the HAL does:
// https://github.com/jonlamb-gh/STM32Cube_FW_F7_V1.8.0/blob/master/Drivers/STM32F7xx_HAL_Driver/Src/stm32f7xx_hal_adc.c#L866
// not sure a cs is needed, since we can't borrow the peripherals anymore?
pub fn adc_irq_handler(_cs: &cortex_m::interrupt::CriticalSection) -> u16 {
    let adc1 = unsafe { &*ADC1::ptr() };
    let data = adc1.dr.read().data().bits();

    // EOCS = 0, but do I need = 1 to catch and save each conversion
    // need to know which channel it is?

    // clear regular channel start flag and end of conversion flag
    adc1.sr
        .modify(|_, w| w.strt().clear_bit().eoc().clear_bit());

    data
}

// TODO - need to enable safe API bits in the HAL crate
fn start_adc(nvic: &mut NVIC) {
    cortex_m::interrupt::free(|_cs| {
        let rcc = unsafe { &*RCC::ptr() };
        let adc1 = unsafe { &*ADC1::ptr() };
        let c_adc = unsafe { &*C_ADC::ptr() };

        // ADC reset and release
        rcc.apb2rstr.modify(|_, w| w.adcrst().set_bit());
        rcc.apb2rstr.modify(|_, w| w.adcrst().clear_bit());

        // enable ADC1/2/3 peripheral clocks
        rcc.apb2enr
            .modify(|_, w| w.adc1en().set_bit().adc2en().set_bit().adc3en().set_bit());

        // set ADC prescaler, PCLK2 divided by 8
        c_adc.ccr.write(|w| unsafe { w.adcpre().bits(0b11) });

        adc1.cr1.write(|w| {
            w
            // disable overrun interrupt
            .ovrie().clear_bit()
            // 12-bit resolution
            .res().bits(0b00)
            // enable scan mode
            .scan().set_bit()
            // disable analog watchdog
            .awden().clear_bit()
            .jawden().clear_bit()
            // enable end of conversion interrupt
            .eocie().set_bit()
        });

        adc1.cr2.write(|w| {
            w
            // right alignment
            .align().clear_bit()
            // EOC set at the end of each sequence
            .eocs().clear_bit()
            // continuous conversion mode
            .cont().set_bit()
            .adon().set_bit()
        });

        // TODO - update this with all AINs
        // sequence length, 2 channels
        adc1.sqr1.write(|w| w.l().bits(0b0001));

        // TODO - ADC_SQRx channel configs IN3, IN10
        // channel conversion sequence in order
        // IN3, IN10
        adc1.sqr3
            .write(|w| unsafe { w.sq1().bits(3).sq2().bits(10) });

        // TODO - ADC_SMPRx - sample time 480 cycles
        // IN3, IN10
        adc1.smpr1.write(|w| unsafe { w.smp10().bits(0b111) });
        adc1.smpr2.write(|w| unsafe { w.smp3().bits(0b111) });

        // power on - moved up to others
        //adc1.cr2.modify(|_, w| w.adon().set_bit());

        // TODO - calibration?

        // start conversion of regular channels
        adc1.cr2.modify(|_, w| w.swstart().set_bit());

        // enable ADC interrupt
        nvic.clear_pending(Interrupt::ADC);
        nvic.enable(Interrupt::ADC);
    });
}
