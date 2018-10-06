//! Panic behavior implementation for OxCC
//!
//! The implementation performs the following (in order):
//! - disable safety/control related GPIO pins
//! - enable red LED and brake lights as a fault indicator
//! - output the PanicInfo to Serial3
//! - `intrinsics::abort`
//!
//! **NOTE**
//! The watchdog is enable by default, so you might not
//! even notice the fault.
//! It might be worth exposing the `hard_fault_indicator()`
//! function so we can call it during initialization if
//! the `ResetConditions` indicate a watchdog reset. That
//! would persist the indication across panicing/faults.
//! We could also disable the watchdog (and other interrupts)
//! completely to stay in the abort.

use core::panic::PanicInfo;
use core::{intrinsics, ptr};
use cortex_m::interrupt::CriticalSection;

use nucleo_f767zi::hal::stm32f7x7;
use nucleo_f767zi::hal::time::*;

struct SerialOutputHandle;

macro_rules! serial_print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        SerialOutputHandle.write_fmt(format_args!($($arg)*)).unwrap();
    });
}

macro_rules! serial_println {
    ($fmt:expr) => (serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (serial_print!(concat!($fmt, "\n"), $($arg)*));
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::free(|cs| {
        disable_controls_gpio(cs);
        hard_fault_indicator(cs);
        serial3_panicinfo_dump(cs, info);
    });

    unsafe { intrinsics::abort() }
}

/// Disable safety/control related GPIO
fn disable_controls_gpio(_cs: &CriticalSection) {
    let gpiod = unsafe { &*stm32f7x7::GPIOD::ptr() };
    let gpioe = unsafe { &*stm32f7x7::GPIOE::ptr() };

    // disable throttle controls spoof-enable pin on PE2
    gpioe.moder.modify(|_, w| w.moder2().output());
    gpioe.odr.modify(|_, w| w.odr2().clear_bit());

    // disable steering controls spoof-enable pin on PD11
    gpiod.moder.modify(|_, w| w.moder11().output());
    gpiod.odr.modify(|_, w| w.odr11().clear_bit());

    // disable brake controls spoof-enable pin on PD12
    gpiod.moder.modify(|_, w| w.moder12().output());
    gpiod.odr.modify(|_, w| w.odr12().clear_bit());
}

/// Enable the hard fault indication
fn hard_fault_indicator(_cs: &CriticalSection) {
    let gpiob = unsafe { &*stm32f7x7::GPIOB::ptr() };
    let gpiod = unsafe { &*stm32f7x7::GPIOD::ptr() };

    // turn red LED (PB14) on
    gpiob.moder.modify(|_, w| w.moder14().output());
    gpiob.odr.modify(|_, w| w.odr14().set_bit());

    // enable the brake lights on PD13
    gpiod.moder.modify(|_, w| w.moder13().output());
    gpiod.odr.modify(|_, w| w.odr13().set_bit());
}

/// Output PanicInfo to Serial3
fn serial3_panicinfo_dump(_cs: &CriticalSection, info: &PanicInfo) {
    let rcc = unsafe { &*stm32f7x7::RCC::ptr() };
    let usart = unsafe { &*stm32f7x7::USART3::ptr() };

    // reset and enable
    rcc.apb1enr.modify(|_, w| w.usart3en().enabled());
    rcc.apb1rstr.modify(|_, w| w.uart3rst().set_bit());
    rcc.apb1rstr.modify(|_, w| w.uart3rst().clear_bit());

    let pclk1: Hertz = Hertz(216_000_000 / 4);
    let baud_rate: Bps = 115_200.bps();
    let brr = pclk1.0 / baud_rate.0;
    usart.brr.write(|w| unsafe { w.bits(brr) });

    // enable USART, tx, rx
    usart.cr1.write(|w| w.ue().set_bit().te().set_bit());

    serial_println!("\n!!! WARNING !!!\n{}", info);
}

fn putchar(byte: u8) -> Result<(), ()> {
    let isr = unsafe { (*stm32f7x7::USART3::ptr()).isr.read() };

    if isr.txe().bit_is_set() && isr.tc().bit_is_set() {
        unsafe { ptr::write_volatile(&(*stm32f7x7::USART3::ptr()).tdr as *const _ as *mut _, byte) }
        Ok(())
    } else {
        Err(())
    }
}

fn flush() -> Result<(), ()> {
    let isr = unsafe { (*stm32f7x7::USART3::ptr()).isr.read() };

    if isr.tc().bit_is_set() {
        Ok(())
    } else {
        Err(())
    }
}

impl ::core::fmt::Write for SerialOutputHandle {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for &b in s.as_bytes() {
            while putchar(b).is_err() {}
            while flush().is_err() {}
        }
        Ok(())
    }
}
