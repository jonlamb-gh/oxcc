//! Fault condition

use dual_signal::{DualSignal, HighLowReader};
use embedded_hal::timer::CountDown;
use nucleo_f767zi::hal::timer::OnePulse;

pub struct FaultCondition<TIMER> {
    monitoring_active: bool,
    timer: TIMER,
}

impl<TIMER> FaultCondition<TIMER>
where
    TIMER: CountDown + OnePulse,
{
    pub fn new(mut timer: TIMER) -> Self {
        timer.reconfigure_one_pulse_mode();

        FaultCondition {
            monitoring_active: false,
            timer,
        }
    }

    pub fn condition_exceeded_duration(&mut self, condition_active: bool) -> bool {
        let mut faulted = false;

        if !condition_active {
            // If a fault condition is not active, update the state to clear
            // the condition active flag and reset the last detection time.
            self.monitoring_active = false;
        } else {
            if !self.monitoring_active {
                // We just detected a condition that may lead to a fault. Update
                // the state to track that the condition is active and store the
                // first time of detection.
                self.monitoring_active = true;
                self.timer.reset();
            }

            if self.timer.wait().is_ok() {
                // The fault condition has been active for longer than the maximum
                // acceptable duration.
                faulted = true;
                self.timer.reset();
            }
        }

        faulted
    }

    pub fn check_voltage_grounded<T: HighLowReader>(&mut self, signal: &DualSignal<T>) -> bool {
        let condition_active = (signal.high() == 0) || (signal.low() == 0);

        self.condition_exceeded_duration(condition_active)
    }
}
