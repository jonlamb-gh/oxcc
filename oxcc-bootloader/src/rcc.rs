use stm32f7::stm32f7x7;

/// Set up PLL to 168MHz from 16MHz HSI
pub fn rcc_init(peripherals: &mut stm32f7x7::Peripherals) {
    let rcc = &peripherals.RCC;
    let flash = &peripherals.FLASH;
    let syscfg = &peripherals.SYSCFG;

    // Reset all peripherals
    rcc.ahb1rstr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });
    rcc.ahb1rstr.write(|w| unsafe { w.bits(0) });
    rcc.ahb2rstr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });
    rcc.ahb2rstr.write(|w| unsafe { w.bits(0) });
    rcc.ahb3rstr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });
    rcc.ahb3rstr.write(|w| unsafe { w.bits(0) });
    rcc.apb1rstr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });
    rcc.apb1rstr.write(|w| unsafe { w.bits(0) });
    rcc.apb2rstr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });
    rcc.apb2rstr.write(|w| unsafe { w.bits(0) });

    // Ensure HSI is on and stable
    rcc.cr.modify(|_, w| w.hsion().set_bit());
    while rcc.cr.read().hsion().bit_is_clear() {}

    // Set system clock to HSI
    rcc.cfgr.modify(|_, w| w.sw().hsi());
    while !rcc.cfgr.read().sws().is_hsi() {}

    // Clear registers to reset value
    rcc.cr.write(|w| w.hsion().set_bit());
    rcc.cfgr.write(|w| unsafe { w.bits(0) });

    // Configure PLL: 16MHz /8 *168 /2, source HSI
    rcc.pllcfgr.write(|w| unsafe {
        w.pllq()
            .bits(4)
            .pllsrc()
            .hsi()
            .pllp()
            .div2()
            .plln()
            .bits(168)
            .pllm()
            .bits(8)
    });
    // Activate PLL
    rcc.cr.modify(|_, w| w.pllon().set_bit());

    // Set other clock domains: PPRE2 to /2, PPRE1 to /4, HPRE to /1
    rcc.cfgr
        .modify(|_, w| w.ppre2().div2().ppre1().div4().hpre().div1());

    // Flash setup: prefetch enabled, 5 wait states (OK for 3.3V
    // at 168MHz)
    flash.acr.write(|w| w.prften().set_bit().latency().bits(5));

    // Swap system clock to PLL
    rcc.cfgr.modify(|_, w| w.sw().pll());
    while !rcc.cfgr.read().sws().is_pll() {}

    // Set SYSCFG early to RMII mode
    rcc.apb2enr.modify(|_, w| w.syscfgen().enabled());
    syscfg.pmc.modify(|_, w| w.mii_rmii_sel().set_bit());

    // Set up peripheral clocks
    rcc.ahb1enr.modify(|_, w| {
        w.gpioaen()
            .enabled()
            .gpioben()
            .enabled()
            .gpiocen()
            .enabled()
            .gpioden()
            .enabled()
            .gpioeen()
            .enabled()
            .gpiogen()
            .enabled()
            .crcen()
            .enabled()
            .ethmacrxen()
            .enabled()
            .ethmactxen()
            .enabled()
            .ethmacen()
            .enabled()
    });
}
