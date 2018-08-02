// https://github.com/jonlamb-gh/oscc/blob/master/firmware/common/libs/fault_check/oscc_check.cpp
// https://github.com/jonlamb-gh/oscc/blob/master/firmware/common/libs/fault_check/oscc_check.h#L19
pub struct FaultCondition {
    monitoring_active: bool,
    condition_start_time: u32,
}

impl FaultCondition {
    pub fn new() -> FaultCondition {
        FaultCondition {
            monitoring_active: false,
            condition_start_time: 0,
        }
    }

    pub fn condition_exceeded_duration(&self, condition_active: bool, max_duration: u32) -> bool {
        false
    }

    pub fn check_voltage_grounded(&self, high: u16, low: u16, max_duration: u32) -> bool {
        false
    }
}
