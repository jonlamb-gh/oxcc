use cortex_m::peripheral::DWT;
use nucleo_f767zi::hal::rcc::Clocks;
use nucleo_f767zi::hal::time::{Instant, MonoTimer};

pub struct MsTimer {
    timer: MonoTimer,
    instant: Instant,
}

impl MsTimer {
    pub fn new(mut dwt: DWT, clocks: Clocks) -> Self {
        let timer = MonoTimer::new(dwt, clocks);

        MsTimer {
            timer,
            instant: timer.now(),
        }
    }

    // TODO - need to convert ticks to time, use timer.frequency()
    pub fn ms(&self) -> u32 {
        self.instant.elapsed()
    }
}
