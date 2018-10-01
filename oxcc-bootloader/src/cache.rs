use stm32f7::stm32f7x7;

/// Enable I and D cache
pub fn cache_enable(core_peripherals: &mut stm32f7x7::CorePeripherals) {
    core_peripherals.SCB.enable_icache();
    core_peripherals
        .SCB
        .enable_dcache(&mut core_peripherals.CPUID);
}
