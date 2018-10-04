//! Peripheral bootstrapping of the OxCC hardware environment

use config;
use cortex_m;
use dac_mcp4922::Mcp4922;
use dac_mcp4922::MODE as DAC_MODE;
use dual_signal::HighLowReader;
use nucleo_f767zi::debug_console::DebugConsole;
use nucleo_f767zi::hal::adc::Adc;
use nucleo_f767zi::hal::adc::Channel as AdcChannel;
use nucleo_f767zi::hal::adc::Prescaler as AdcPrescaler;
use nucleo_f767zi::hal::adc::Resolution as AdcResolution;
use nucleo_f767zi::hal::adc::SampleTime as AdcSampleTime;
use nucleo_f767zi::hal::can::Can;
use nucleo_f767zi::hal::iwdg::{Iwdg, IwdgConfig, WatchdogTimeout};
use nucleo_f767zi::hal::prelude::*;
use nucleo_f767zi::hal::rcc::ResetConditions;
use nucleo_f767zi::hal::serial::Serial;
use nucleo_f767zi::hal::spi::Spi;
use nucleo_f767zi::hal::stm32f7x7;
use nucleo_f767zi::hal::stm32f7x7::{ADC1, ADC2, ADC3, IWDG};
use nucleo_f767zi::led::{Color, Leds};
use nucleo_f767zi::UserButtonPin;
use vehicle::FAULT_HYSTERESIS;

pub use types::*;

/// Control module CAN report frame publish rate
///
/// Brake, throttle and steering modules will publish
/// their report frames to the control CAN bus at
/// this rate.
pub const CAN_PUBLISH_HZ: u32 = 50;

/// ADC configuration
///
/// The ADC(s) are configured to match the original OSCC
/// Arduino resolution of 10 bits instead of the full 12 bit
/// resolution.
/// This allows existing vehicle configurations to be used without
/// re-calibrating.
///
/// The OSCC Arduino builds were likely using the default ADC
/// configuration which would give about 104 microseconds per
/// `analogRead()`.
///
/// OxCC configures the ADC(s) such that the conversion time
/// is about 18.26 microseconds.
///
/// Total conversion time = sampling time + 13 cycles for 10-bit resolution
///   - APB2 clock = 216 MHz / 2
///   - ADC clock = APB2 clock / prescaler = 108 / Prescaler4 = 27 MHz
///   - conversion time = Cycles480 + 13 = 493 cycles == 18.26 us
///
/// **NOTE**
/// Prescalers must be chosen such that the ADC clock
/// does not exceed 30 MHz
pub const ADC_PRESCALER: AdcPrescaler = AdcPrescaler::Prescaler4;
pub const ADC_SAMPLE_TIME: AdcSampleTime = AdcSampleTime::Cycles480;
pub const ADC_RESOLUTION: AdcResolution = AdcResolution::Bits10;

/// Number of analog conversion samples read
/// by the DAC signal discontinuity mechanism.
///
/// **NOTE**
/// This is likely the result of poor ADC hardware on the original
/// OSCC Arduino hardware. I suspect we can get rid of it now that
/// we're using the full conversion time of Cycles480 which is
/// pretty stable.
pub const DAC_SAMPLE_AVERAGE_COUNT: u32 = 20;

pub struct FullBoard {
    pub debug_console: DebugConsole,
    pub leds: Leds,
    pub user_button: UserButtonPin,
    pub can_publish_timer: CanPublishTimer,
    pub wdg: Iwdg<IWDG>,
    pub reset_conditions: ResetConditions,
    control_can: ControlCan,
    obd_can: ObdCan,
    brake_pedal_position_sensor: BrakePedalPositionSensor,
    accelerator_position_sensor: AcceleratorPositionSensor,
    torque_sensor: TorqueSensor,
    brake_dac: BrakeDac,
    throttle_dac: ThrottleDac,
    steering_dac: SteeringDac,
    brake_pins: BrakePins,
    throttle_pins: ThrottlePins,
    steering_pins: SteeringPins,
    brake_grounded_fault_timer: BrakeGroundedFaultTimer,
    brake_override_fault_timer: BrakeOverrideFaultTimer,
    throttle_grounded_fault_timer: ThrottleGroundedFaultTimer,
    throttle_override_fault_timer: ThrottleOverrideFaultTimer,
    steering_grounded_fault_timer: SteeringGroundedFaultTimer,
}

pub struct Board {
    pub leds: Leds,
    pub user_button: UserButtonPin,
    pub wdg: Iwdg<IWDG>,
    pub reset_conditions: ResetConditions,
}

impl FullBoard {
    pub fn new() -> Self {
        // read the RCC reset condition flags before anything else
        let reset_conditions = ResetConditions::read_and_clear();

        let mut core_peripherals =
            cortex_m::Peripherals::take().expect("Failed to take cortex_m::Peripherals");
        let peripherals =
            stm32f7x7::Peripherals::take().expect("Failed to take stm32f7x7::Peripherals");

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
        let mut gpioe = peripherals.GPIOE.split(&mut rcc.ahb1);
        let mut gpiof = peripherals.GPIOF.split(&mut rcc.ahb1);

        let brake_pins = BrakePins {
            spoof_enable: gpiod
                .pd12
                .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper),
            brake_light_enable: gpiod
                .pd13
                .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper),
            pedal_pos_sensor_high: gpioa
                .pa3
                .into_analog_input(&mut gpioa.moder, &mut gpioa.pupdr),
            pedal_pos_sensor_low: gpioc
                .pc0
                .into_analog_input(&mut gpioc.moder, &mut gpioc.pupdr),
        };

        // TODO - move these once DAC impl is ready
        let brake_sck: BrakeSpiSckPin = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let brake_miso: BrakeSpiMisoPin = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let brake_mosi: BrakeSpiMosiPin = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let brake_nss: BrakeSpiNssPin = gpioa
            .pa4
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

        let throttle_pins = ThrottlePins {
            spoof_enable: gpioe
                .pe2
                .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            accel_pos_sensor_high: gpioc
                .pc3
                .into_analog_input(&mut gpioc.moder, &mut gpioc.pupdr),
            accel_pos_sensor_low: gpiob
                .pb1
                .into_analog_input(&mut gpiob.moder, &mut gpiob.pupdr),
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
            torque_sensor_high: gpiof
                .pf5
                .into_analog_input(&mut gpiof.moder, &mut gpiof.pupdr),
            torque_sensor_low: gpiof
                .pf10
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
        // let clocks = rcc.cfgr.freeze(&mut flash.acr);

        // TODO - enable OverDrive to get 216 MHz
        // configure maximum clock frequency at 200 MHz
        let clocks = rcc.cfgr.freeze_max(&mut flash.acr);

        // TODO - this can be moved into the HAL once it's aware of the clocks
        let adc_clock = match ADC_PRESCALER {
            AdcPrescaler::Prescaler2 => clocks.pclk2().0 / 2,
            AdcPrescaler::Prescaler4 => clocks.pclk2().0 / 4,
            AdcPrescaler::Prescaler6 => clocks.pclk2().0 / 6,
            AdcPrescaler::Prescaler8 => clocks.pclk2().0 / 8,
        };
        assert!(adc_clock <= 30_000_000);

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

        // NOTE: the default config can fail if there are CAN bus or config issues
        // &CanConfig::default(),
        // loopback/silent mode can be used for testing
        // &CanConfig { loopback_mode: true, silent_mode: true,
        // ..CanConfig::default() },
        let control_can = Can::can1(
            peripherals.CAN1,
            (can1_tx, can1_rx),
            &mut rcc.apb1,
            &config::CONTROL_CAN_CONFIG,
        ).expect("Failed to configure control CAN (CAN1)");

        let obd_can = Can::can2(
            peripherals.CAN2,
            (can2_tx, can2_rx),
            &mut rcc.apb1,
            &config::OBD_CAN_CONFIG,
        ).expect("Failed to configure OBD CAN (CAN2)");

        // apply control CAN filters
        for filter in &config::gather_control_can_filters() {
            control_can
                .configure_filter(&filter)
                .expect("Failed to configure control CAN filter");
        }

        // apply OBD CAN filters
        for filter in &config::gather_obd_can_filters() {
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

        FullBoard {
            debug_console: DebugConsole::new(serial),
            leds,
            user_button: gpioc
                .pc13
                .into_pull_down_input(&mut gpioc.moder, &mut gpioc.pupdr),
            can_publish_timer: CanPublishTimer::tim2(
                peripherals.TIM2,
                CAN_PUBLISH_HZ.hz(),
                clocks,
                &mut rcc.apb1,
            ),
            wdg: Iwdg::new(
                peripherals.IWDG,
                IwdgConfig::from(WatchdogTimeout::Wdto50ms),
            ),
            reset_conditions,
            control_can,
            obd_can,
            brake_pedal_position_sensor: BrakePedalPositionSensor {
                adc1: Adc::adc1(
                    peripherals.ADC1,
                    &mut c_adc,
                    &mut rcc.apb2,
                    ADC_PRESCALER,
                    ADC_RESOLUTION,
                ),
            },
            accelerator_position_sensor: AcceleratorPositionSensor {
                adc2: Adc::adc2(
                    peripherals.ADC2,
                    &mut c_adc,
                    &mut rcc.apb2,
                    ADC_PRESCALER,
                    ADC_RESOLUTION,
                ),
            },
            torque_sensor: TorqueSensor {
                adc3: Adc::adc3(
                    peripherals.ADC3,
                    &mut c_adc,
                    &mut rcc.apb2,
                    ADC_PRESCALER,
                    ADC_RESOLUTION,
                ),
            },
            brake_dac: Mcp4922::new(brake_spi, brake_nss),
            throttle_dac: Mcp4922::new(throttle_spi, throttle_nss),
            steering_dac: Mcp4922::new(steering_spi, steering_nss),
            brake_pins,
            throttle_pins,
            steering_pins,
            brake_grounded_fault_timer: BrakeGroundedFaultTimer::tim3(
                peripherals.TIM3,
                (1000 / FAULT_HYSTERESIS).hz(),
                clocks,
                &mut rcc.apb1,
            ),
            brake_override_fault_timer: BrakeOverrideFaultTimer::tim4(
                peripherals.TIM4,
                (1000 / FAULT_HYSTERESIS).hz(),
                clocks,
                &mut rcc.apb1,
            ),
            throttle_grounded_fault_timer: ThrottleGroundedFaultTimer::tim5(
                peripherals.TIM5,
                (1000 / FAULT_HYSTERESIS).hz(),
                clocks,
                &mut rcc.apb1,
            ),
            throttle_override_fault_timer: ThrottleOverrideFaultTimer::tim6(
                peripherals.TIM6,
                (1000 / FAULT_HYSTERESIS).hz(),
                clocks,
                &mut rcc.apb1,
            ),
            steering_grounded_fault_timer: SteeringGroundedFaultTimer::tim7(
                peripherals.TIM7,
                (1000 / FAULT_HYSTERESIS).hz(),
                clocks,
                &mut rcc.apb1,
            ),
        }
    }

    pub fn split_components(
        self,
    ) -> (
        Board,
        BrakeDac,
        BrakePins,
        BrakePedalPositionSensor,
        BrakeGroundedFaultTimer,
        BrakeOverrideFaultTimer,
        AcceleratorPositionSensor,
        ThrottleDac,
        ThrottlePins,
        ThrottleGroundedFaultTimer,
        ThrottleOverrideFaultTimer,
        TorqueSensor,
        SteeringDac,
        SteeringPins,
        SteeringGroundedFaultTimer,
        DebugConsole,
        CanPublishTimer,
        ControlCan,
        ObdCan,
    ) {
        let FullBoard {
            debug_console,
            leds,
            user_button,
            can_publish_timer,
            wdg,
            reset_conditions,
            control_can,
            obd_can,
            brake_pedal_position_sensor,
            accelerator_position_sensor,
            torque_sensor,
            brake_dac,
            throttle_dac,
            steering_dac,
            brake_pins,
            throttle_pins,
            steering_pins,
            brake_grounded_fault_timer,
            brake_override_fault_timer,
            throttle_grounded_fault_timer,
            throttle_override_fault_timer,
            steering_grounded_fault_timer,
        } = self;
        (
            Board {
                leds,
                user_button,
                wdg,
                reset_conditions,
            },
            brake_dac,
            brake_pins,
            brake_pedal_position_sensor,
            brake_grounded_fault_timer,
            brake_override_fault_timer,
            accelerator_position_sensor,
            throttle_dac,
            throttle_pins,
            throttle_grounded_fault_timer,
            throttle_override_fault_timer,
            torque_sensor,
            steering_dac,
            steering_pins,
            steering_grounded_fault_timer,
            debug_console,
            can_publish_timer,
            control_can,
            obd_can,
        )
    }
}

impl Board {
    pub fn user_button(&mut self) -> bool {
        self.user_button.is_high()
    }
}

// brake module owns ADC1
pub struct BrakePedalPositionSensor {
    adc1: Adc<ADC1>,
}

impl HighLowReader for BrakePedalPositionSensor {
    fn read_high(&self) -> u16 {
        self.adc1.read(AdcChannel::Adc123In3, ADC_SAMPLE_TIME)
    }
    fn read_low(&self) -> u16 {
        self.adc1.read(AdcChannel::Adc123In10, ADC_SAMPLE_TIME)
    }
}

// throttle module owns ADC2
pub struct AcceleratorPositionSensor {
    adc2: Adc<ADC2>,
}

impl HighLowReader for AcceleratorPositionSensor {
    fn read_high(&self) -> u16 {
        self.adc2.read(AdcChannel::Adc123In13, ADC_SAMPLE_TIME)
    }
    fn read_low(&self) -> u16 {
        self.adc2.read(AdcChannel::Adc12In9, ADC_SAMPLE_TIME)
    }
}

// steering module owns ADC3
pub struct TorqueSensor {
    adc3: Adc<ADC3>,
}

impl HighLowReader for TorqueSensor {
    fn read_high(&self) -> u16 {
        self.adc3.read(AdcChannel::Adc3In15, ADC_SAMPLE_TIME)
    }
    fn read_low(&self) -> u16 {
        self.adc3.read(AdcChannel::Adc3In8, ADC_SAMPLE_TIME)
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
