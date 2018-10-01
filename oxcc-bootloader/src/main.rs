#![no_std]
#![no_main]

extern crate byteorder;
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate cortex_m_semihosting;
extern crate oxcc_bootloader_lib;
extern crate panic_abort;
// Can be useful for debugging
//extern crate panic_semihosting;
extern crate smoltcp;
extern crate stm32f7;

use cortex_m_rt::{entry, exception, ExceptionFrame};
use stm32f7::stm32f7x7;

/// Try to print over semihosting if a debugger is available
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        use cortex_m;
        use cortex_m_semihosting;
        if unsafe { (*cortex_m::peripheral::DCB::ptr()).dhcsr.read() & 1 == 1 } {
            if let Ok(mut stdout) = cortex_m_semihosting::hio::hstdout() {
                write!(stdout, $($arg)*).ok();
            }
        }
    })
}

/// Try to print a line over semihosting if a debugger is available
#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

mod bootload;
mod cache;
mod config;
mod ethernet;
mod flash;
mod gpio;
mod network;
mod rcc;
mod systick;

use cache::cache_enable;
use gpio::gpio_init;
use oxcc_bootloader_lib::{Error, Result};
use rcc::rcc_init;
use systick::systick_init;

// Pull in build information (from `built` crate)
mod build_info {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[entry]
fn main() -> ! {
    let mut peripherals = stm32f7x7::Peripherals::take().unwrap();
    let mut core_peripherals = stm32f7x7::CorePeripherals::take().unwrap();

    // Jump to user code if it exists and hasn't asked us to run
    if let Some(address) = flash::valid_user_code() {
        if !config::should_enter_bootloader(&mut peripherals) {
            bootload::bootload(&mut core_peripherals.SCB, address);
        }
    }

    println!("");
    println!("|-=-=-=-=-=-=-= 0xCC Bootloader =-=-=-=-=-=-=-");
    println!(
        "| Version {} {}",
        build_info::PKG_VERSION,
        build_info::GIT_VERSION.unwrap()
    );
    println!("| Platform {}", build_info::TARGET);
    println!("| Built on {}", build_info::BUILT_TIME_UTC);
    println!("| {}", build_info::RUSTC_VERSION);
    println!("|-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-\n");

    print!(" Initialising cache...                ");
    cache_enable(&mut core_peripherals);
    println!("OK");

    print!(" Initialising clocks...               ");
    rcc_init(&mut peripherals);
    println!("OK");

    print!(" Initialising GPIOs...                ");
    gpio_init(&mut peripherals);
    println!("OK");

    print!(" Reading configuration...             ");
    let cfg = match flash::UserConfig::get(&mut peripherals.CRC) {
        Some(cfg) => {
            println!("OK");
            cfg
        }
        None => {
            println!("Err\nCouldn't read configuration, using default.");
            flash::DEFAULT_CONFIG
        }
    };
    println!("{}", cfg);
    let mac_addr = smoltcp::wire::EthernetAddress::from_bytes(&cfg.mac_address);

    print!(" Initialising Ethernet...             ");
    let mut ethdev =
        ethernet::EthernetDevice::new(peripherals.ETHERNET_MAC, peripherals.ETHERNET_DMA);
    ethdev.init(&mut peripherals.RCC, mac_addr);
    println!("OK");

    print!(" Waiting for link...                  ");
    ethdev.block_until_link();
    println!("OK");

    print!(" Initialising network...              ");
    let ip_addr = smoltcp::wire::Ipv4Address::from_bytes(&cfg.ip_address);
    let ip_cidr = smoltcp::wire::Ipv4Cidr::new(ip_addr, cfg.ip_prefix);
    let cidr = smoltcp::wire::IpCidr::Ipv4(ip_cidr);
    network::init(ethdev, mac_addr, cidr);
    println!("OK");

    // Move flash peripheral into flash module
    flash::init(peripherals.FLASH);

    // TODO - Blink status LED?
    println!(" Ready.\n");

    // Begin periodic tasks via systick
    systick_init(&mut core_peripherals.SYST);

    loop {
        cortex_m::asm::wfi();
    }
}

static mut SYSTICK_TICKS: u32 = 0;
static mut SYSTICK_RESET_AT: Option<u32> = None;

#[exception]
fn SysTick() {
    let ticks = unsafe { core::ptr::read_volatile(&SYSTICK_TICKS) + 1 };
    unsafe { core::ptr::write_volatile(&mut SYSTICK_TICKS, ticks) };
    network::poll(i64::from(ticks));
    if let Some(reset_time) = unsafe { core::ptr::read_volatile(&SYSTICK_RESET_AT) } {
        if ticks >= reset_time {
            println!("Performing scheduled reset");
            bootload::reset_to_user_firmware();
        }
    }
}

/// Reset after some ms delay.
pub fn schedule_reset(delay: u32) {
    cortex_m::interrupt::free(|_| unsafe {
        let ticks = core::ptr::read_volatile(&SYSTICK_TICKS) + delay;
        core::ptr::write_volatile(&mut SYSTICK_RESET_AT, Some(ticks));
    });
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
