// https://github.com/jonlamb-gh/oscc/blob/master/firmware/common/libs/fault_check/oscc_check.cpp
// https://github.com/jonlamb-gh/oscc/blob/master/firmware/common/libs/fault_check/oscc_check.h#L19

use dual_signal::{DualSignal, HighLowReader};
use ms_timer::MsTimer;

pub struct FaultCondition {
    monitoring_active: bool,
    condition_start_time: u32,
}

impl FaultCondition {
    pub const fn new() -> Self {
        FaultCondition {
            monitoring_active: false,
            condition_start_time: 0,
        }
    }

    pub fn condition_exceeded_duration(
        &mut self,
        condition_active: bool,
        max_duration: u32,
        timer_ms: &MsTimer,
    ) -> bool {
        let mut faulted = false;

        if !condition_active {
            /*
             * If a fault condition is not active, update the state to clear
             * the condition active flag and reset the last detection time.
             */
            self.monitoring_active = false;
            self.condition_start_time = 0;
        } else {
            let now = timer_ms.ms();

            if !self.monitoring_active {
                /* We just detected a condition that may lead to a fault. Update
                 * the state to track that the condition is active and store the
                 * first time of detection.
                 */
                self.monitoring_active = true;
                self.condition_start_time = now;
            }

            // TODO - need to fix this ported logic
            // panicked at 'attempt to subtract with overflow', src/fault_condition.rs:47:28
            let duration = now - self.condition_start_time;

            if duration >= max_duration {
                /* The fault condition has been active for longer than the maximum
                 * acceptable duration.
                 */
                faulted = true;
            }
        }

        faulted
    }

    pub fn check_voltage_grounded<T: HighLowReader>(
        &mut self,
        signal: &DualSignal<T>,
        max_duration: u32,
        timer_ms: &MsTimer,
    ) -> bool {
        let condition_active = (signal.high() == 0) || (signal.low() == 0);

        self.condition_exceeded_duration(condition_active, max_duration, timer_ms)
    }
}
