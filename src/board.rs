use adc_signal::AdcSignal;
use config;
use cortex_m;
use dac_mcp4922::Mcp4922;
use dac_mcp4922::MODE as DAC_MODE;
use ms_timer::MsTimer;
use nucleo_f767zi::debug_console::DebugConsole;
use nucleo_f767zi::hal::adc::{Adc, AdcChannel, AdcSampleTime};
use nucleo_f767zi::hal::can::Can;
use nucleo_f767zi::hal::delay::Delay;
use nucleo_f767zi::hal::iwdg::{Iwdg, Prescaler};
use nucleo_f767zi::hal::prelude::*;
use nucleo_f767zi::hal::rcc::ResetConditions;
use nucleo_f767zi::hal::serial::Serial;
use nucleo_f767zi::hal::spi::Spi;
use nucleo_f767zi::hal::stm32f7x7;
use nucleo_f767zi::hal::stm32f7x7::{ADC1, ADC3, IWDG};
use nucleo_f767zi::led::{Color, Leds};
use nucleo_f767zi::UserButtonPin;

pub use types::*;

// feature to pick how to route up debug_println/println?
// or
// - println! -> Serial3 console (currently debug_console)
// - debug_println! -> ITM/semihosting link

pub const CAN_PUBLISH_HZ: u32 = 50;

pub const ADC_SAMPLE_TIME: AdcSampleTime = AdcSampleTime::Cycles480;

// not sure if the averaging is needed, we might be able to just use a
// single read with large Cycles480 sample time?
pub const DAC_SAMPLE_AVERAGE_COUNT: u32 = 20;

pub struct Board {
    pub debug_console: DebugConsole,
    pub leds: Leds,
    pub user_button: UserButtonPin,
    pub delay: Delay,
    pub timer_ms: MsTimer,
    pub can_publish_timer: CanPublishTimer,
    pub wdg: Iwdg<IWDG>,
    pub reset_conditions: ResetConditions,
    control_can: ControlCan,
    obd_can: ObdCan,
    adc1: Adc<ADC1>,
    adc3: Adc<ADC3>,
    brake_dac: BrakeDac,
    throttle_dac: ThrottleDac,
    steering_dac: SteeringDac,
    brake_pins: BrakePins,
    throttle_pins: ThrottlePins,
    steering_pins: SteeringPins,
}

impl Board {
    pub fn new() -> Self {
        // read the RCC reset condition flags before anything else
        let reset_conditions = ResetConditions::read_and_clear();

        let mut core_peripherals = cortex_m::Peripherals::take().unwrap();
        let peripherals = stm32f7x7::Peripherals::take().unwrap();

        core_peripherals.SCB.enable_icache();
        core_peripherals
            .SCB
            .enable_dcache(&mut core_peripherals.CPUID);

        let mut flash = peripherals.FLASH.constrain();
        let mut rcc = peripherals.RCC.constrain();
        let mut c_adc = peripherals.C_ADC;

        let mut gpiob = peripherals.GPIOB.split(&mut rcc.ahb1);
        let mut gpioa = peripherals.GPIOA.split(&mut rcc.ahb1);
        let mut gpioc = peripherals.GPIOC.split(&mut rcc.ahb1);
        let mut gpiod = peripherals.GPIOD.split(&mut rcc.ahb1);
        let mut gpiof = peripherals.GPIOF.split(&mut rcc.ahb1);

        let brake_pins = BrakePins {
            spoof_enable: gpiod
                .pd12
                .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper),
            brake_light_enable: gpiod
                .pd13
                .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper),
            pedal_pos_sensor_high: gpiof
                .pf5
                .into_analog_input(&mut gpiof.moder, &mut gpiof.pupdr),
            pedal_pos_sensor_low: gpiof
                .pf10
                .into_analog_input(&mut gpiof.moder, &mut gpiof.pupdr),
        };

        // TODO - move these once DAC impl is ready
        let brake_sck: BrakeSpiSckPin = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let brake_miso: BrakeSpiMisoPin = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let brake_mosi: BrakeSpiMosiPin = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let brake_nss: BrakeSpiNssPin = gpioa
            .pa4
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

        let throttle_pins = ThrottlePins {
            spoof_enable: gpiod
                .pd10
                .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper),
            accel_pos_sensor_high: gpioa
                .pa3
                .into_analog_input(&mut gpioa.moder, &mut gpioa.pupdr),
            accel_pos_sensor_low: gpioc
                .pc0
                .into_analog_input(&mut gpioc.moder, &mut gpioc.pupdr),
        };

        let throttle_sck: ThrottleSpiSckPin =
            gpiob.pb10.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
        let throttle_miso: ThrottleSpiMisoPin =
            gpioc.pc2.into_af5(&mut gpioc.moder, &mut gpioc.afrl);
        let throttle_mosi: ThrottleSpiMosiPin =
            gpiob.pb15.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
        let throttle_nss: ThrottleSpiNssPin = gpiob
            .pb4
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

        let steering_pins = SteeringPins {
            spoof_enable: gpiod
                .pd11
                .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper),
            torque_sensor_high: gpioc
                .pc3
                .into_analog_input(&mut gpioc.moder, &mut gpioc.pupdr),
            torque_sensor_low: gpiof
                .pf3
                .into_analog_input(&mut gpiof.moder, &mut gpiof.pupdr),
        };

        let steering_sck: SteeringSpiSckPin =
            gpioc.pc10.into_af5(&mut gpioc.moder, &mut gpioc.afrh);
        let steering_miso: SteeringSpiMisoPin =
            gpioc.pc11.into_af5(&mut gpioc.moder, &mut gpioc.afrh);
        let steering_mosi: SteeringSpiMosiPin =
            gpioc.pc12.into_af5(&mut gpioc.moder, &mut gpioc.afrh);
        let steering_nss: SteeringSpiNssPin = gpioa
            .pa15
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

        let led_r = gpiob
            .pb14
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
        let led_g = gpiob
            .pb0
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
        let led_b = gpiob
            .pb7
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

        let usart3_tx = gpiod.pd8.into_af7(&mut gpiod.moder, &mut gpiod.afrh);
        let usart3_rx = gpiod.pd9.into_af7(&mut gpiod.moder, &mut gpiod.afrh);

        let can1_tx = gpiod.pd1.into_af9(&mut gpiod.moder, &mut gpiod.afrl);
        let can1_rx = gpiod.pd0.into_af9(&mut gpiod.moder, &mut gpiod.afrl);

        let can2_tx = gpiob.pb13.into_af9(&mut gpiob.moder, &mut gpiob.afrh);
        let can2_rx = gpiob.pb12.into_af9(&mut gpiob.moder, &mut gpiob.afrh);

        // default clock configuration runs at 16 MHz
        //let clocks = rcc.cfgr.freeze(&mut flash.acr);

        // TODO - enable OverDrive to get 216 MHz
        // configure maximum clock frequency at 200 MHz
        let clocks = rcc.cfgr.freeze_max(&mut flash.acr);

        // TODO - use the safe APIs once this block solidifies
        unsafe {
            // TODO - move this constant into BSP crate?
            // unlock registers to enable DWT cycle counter for MsTimer
            core_peripherals.DWT.lar.write(0xC5ACCE55);
        }

        let mut leds = Leds::new(led_r, led_g, led_b);
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

        /* NOTE: the default config can fail if there are CAN bus or config issues */
        /* &CanConfig::default(), */
        /* loopback/silent mode can be used for testing */
        /* &CanConfig { loopback_mode: true, silent_mode: true,
         * ..CanConfig::default() }, */
        let control_can = Can::can1(
            peripherals.CAN1,
            (can1_tx, can1_rx),
            &mut rcc.apb1,
            &config::CONTROL_CAN_CONFIG,
        ).expect("Failed to configure ontrol CAN (CAN1)");

        let obd_can = Can::can2(
            peripherals.CAN2,
            (can2_tx, can2_rx),
            &mut rcc.apb1,
            &config::OBD_CAN_CONFIG,
        ).expect("Failed to configure OBD CAN (CAN2)");

        // apply control CAN filters
        for filter in config::gather_control_can_filters().iter() {
            control_can
                .configure_filter(&filter)
                .expect("Failed to configure control CAN filter");
        }

        // apply OBD CAN filters
        for filter in config::gather_obd_can_filters().iter() {
            obd_can
                .configure_filter(&filter)
                .expect("Failed to configure OBD CAN filter");
        }

        let brake_spi: BrakeSpi = Spi::spi1(
            peripherals.SPI1,
            (brake_sck, brake_miso, brake_mosi),
            DAC_MODE,
            1.mhz().into(),
            clocks,
            &mut rcc.apb2,
        );

        let throttle_spi: ThrottleSpi = Spi::spi2(
            peripherals.SPI2,
            (throttle_sck, throttle_miso, throttle_mosi),
            DAC_MODE,
            1.mhz().into(),
            clocks,
            &mut rcc.apb1,
        );

        let steering_spi: SteeringSpi = Spi::spi3(
            peripherals.SPI3,
            (steering_sck, steering_miso, steering_mosi),
            DAC_MODE,
            1.mhz().into(),
            clocks,
            &mut rcc.apb1,
        );

        Board {
            debug_console: DebugConsole::new(serial),
            leds,
            user_button: gpioc
                .pc13
                .into_pull_down_input(&mut gpioc.moder, &mut gpioc.pupdr),
            delay: Delay::new(core_peripherals.SYST, clocks),
            timer_ms: MsTimer::new(core_peripherals.DWT, clocks),
            can_publish_timer: CanPublishTimer::tim2(
                peripherals.TIM2,
                CAN_PUBLISH_HZ.hz(),
                clocks,
                &mut rcc.apb1,
            ),
            // TODO - use LSI oscillator frequency to get units in time
            wdg: Iwdg::new(peripherals.IWDG, 0xF, Prescaler::Prescaler64),
            reset_conditions,
            control_can,
            obd_can,
            adc1: Adc::adc1(peripherals.ADC1, &mut c_adc, &mut rcc.apb2),
            adc3: Adc::adc3(peripherals.ADC3, &mut c_adc, &mut rcc.apb2),
            brake_dac: Mcp4922::new(brake_spi, brake_nss),
            throttle_dac: Mcp4922::new(throttle_spi, throttle_nss),
            steering_dac: Mcp4922::new(steering_spi, steering_nss),
            brake_pins,
            throttle_pins,
            steering_pins,
        }
    }

    pub fn user_button(&mut self) -> bool {
        self.user_button.is_high()
    }

    pub fn brake_spoof_enable(&mut self) -> &mut BrakeSpoofEnablePin {
        &mut self.brake_pins.spoof_enable
    }

    pub fn brake_light_enable(&mut self) -> &mut BrakeLightEnablePin {
        &mut self.brake_pins.brake_light_enable
    }

    pub fn throttle_spoof_enable(&mut self) -> &mut ThrottleSpoofEnablePin {
        &mut self.throttle_pins.spoof_enable
    }

    pub fn steering_spoof_enable(&mut self) -> &mut SteeringSpoofEnablePin {
        &mut self.steering_pins.spoof_enable
    }

    pub fn control_can(&mut self) -> &mut ControlCan {
        &mut self.control_can
    }

    pub fn obd_can(&mut self) -> &mut ObdCan {
        &mut self.obd_can
    }

    pub fn brake_dac(&mut self) -> &mut BrakeDac {
        &mut self.brake_dac
    }

    pub fn throttle_dac(&mut self) -> &mut ThrottleDac {
        &mut self.throttle_dac
    }

    pub fn steering_dac(&mut self) -> &mut SteeringDac {
        &mut self.steering_dac
    }

    pub fn analog_read(&mut self, signal: AdcSignal, sample_time: AdcSampleTime) -> u16 {
        let channel = AdcChannel::from(signal);
        match signal {
            AdcSignal::AcceleratorPositionSensorHigh => self.adc1.read(channel, sample_time),
            AdcSignal::AcceleratorPositionSensorLow => self.adc1.read(channel, sample_time),
            AdcSignal::TorqueSensorHigh => self.adc1.read(channel, sample_time),
            AdcSignal::TorqueSensorLow => self.adc3.read(channel, sample_time),
            AdcSignal::BrakePedalPositionSensorHigh => self.adc3.read(channel, sample_time),
            AdcSignal::BrakePedalPositionSensorLow => self.adc3.read(channel, sample_time),
        }
    }
}

pub fn hard_fault_indicator() {
    cortex_m::interrupt::free(|_cs| unsafe {
        let peripherals = stm32f7x7::Peripherals::steal();
        let mut rcc = peripherals.RCC.constrain();
        let mut gpiob = peripherals.GPIOB.split(&mut rcc.ahb1);

        let led_r = gpiob
            .pb14
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
        let led_g = gpiob
            .pb0
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
        let led_b = gpiob
            .pb7
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

        let mut leds = Leds::new(led_r, led_g, led_b);
        leds[Color::Red].on();
    });
}
