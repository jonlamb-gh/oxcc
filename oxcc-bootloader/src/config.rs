//! Chip and board specific configuration settings go here.
use bootload;
use stm32f7x7;

/// TCP port to listen on
pub const TCP_PORT: u16 = 7776;

/// PHY address
pub const ETH_PHY_ADDR: u8 = 0;

/// Start address of each sector in flash
pub const FLASH_SECTOR_ADDRESSES: [u32; 12] = [
    0x0800_0000,
    0x0800_8000,
    0x0801_0000,
    0x0801_8000,
    0x0802_0000,
    0x0804_0000,
    0x0808_0000,
    0x080C_0000,
    0x0810_0000,
    0x0814_0000,
    0x0818_0000,
    0x081C_0000,
];

/// Final valid address in flash
pub const FLASH_END: u32 = 0x081F_FFFF;

/// Address of configuration sector. Must be one of the start addresses in
/// FLASH_SECTOR_ADDRESSES.
pub const FLASH_CONFIG: u32 = FLASH_SECTOR_ADDRESSES[3];

/// Address of user firmware sector. Must be one of the start addresses in
/// FLASH_SECTOR_ADDRESSES.
pub const FLASH_USER: u32 = FLASH_SECTOR_ADDRESSES[4];

/// Magic value used in this module to check if bootloader should start.
pub const BOOTLOAD_FLAG_VALUE: u32 = 0xB00110AD;
/// Address of magic value used in this module to check if bootloader should
/// start.
/// SRAM1 starts at 0x2002_0000
/// DTCM RAM starts at 0x2000_0000
pub const BOOTLOAD_FLAG_ADDRESS: u32 = 0x2000_0000;

/// This function should return true if the bootloader should enter bootload
/// mode, or false to immediately chainload the user firmware.
///
/// By default we check if there was a software reset and a magic value is set
/// in RAM, but you could also check GPIOs etc here.
///
/// Ensure any state change to the peripherals is reset before returning from
/// this function.
pub fn should_enter_bootloader(peripherals: &mut stm32f7x7::Peripherals) -> bool {
    // Our plan is:
    // * If the reset was a software reset, and the magic flag is in the magic
    // location, then the user firmware requested bootload, so enter bootload.
    //
    // * Otherwise we check if PC13 (user-button) is HIGH for at least a
    // full byte period of the UART
    let cond1 = bootload::was_software_reset(&mut peripherals.RCC) && bootload::flag_set();

    // User button on PC13, pull-down/active-high
    peripherals.RCC.ahb1enr.modify(|_, w| w.gpiocen().enabled());
    peripherals.GPIOC.moder.modify(|_, w| w.moder13().input());
    peripherals
        .GPIOC
        .pupdr
        .modify(|_, w| w.pupdr13().pull_down());

    let hsi_clk = 16_000_000;
    let sync_baud = 1_000_000;
    let bit_periods = 10;
    let delay = (hsi_clk / sync_baud) * bit_periods;
    let mut cond2 = true;
    for _ in 0..delay {
        cond2 &= peripherals.GPIOC.idr.read().idr13().bit_is_set();
    }

    peripherals
        .RCC
        .ahb1enr
        .modify(|_, w| w.gpiocen().disabled());
    cond1 || cond2
}
