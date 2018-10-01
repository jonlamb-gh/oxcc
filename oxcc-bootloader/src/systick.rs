use cortex_m;
use stm32f7::stm32f7x7;

/// Set up the systick to provide a 1ms timebase
pub fn systick_init(syst: &mut stm32f7x7::SYST) {
    syst.set_reload((168_000_000 / 8) / 1000);
    syst.clear_current();
    syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::External);
    syst.enable_interrupt();
    syst.enable_counter();
}
