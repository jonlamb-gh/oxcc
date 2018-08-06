// https://github.com/jonlamb-gh/oscc/blob/master/firmware/common/libs/pid/oscc_pid.h
// https://github.com/jonlamb-gh/oscc/blob/master/firmware/common/libs/pid/oscc_pid.cpp

pub struct Pid {
    windup_guard: f32,
    proportional_gain: f32,
    integral_gain: f32,
    derivative_gain: f32,
    prev_input: f32,
    int_error: f32,
    control: f32,
    prev_steering_angle: f32,
}

impl Pid {
    pub fn new() -> Pid {
        Pid {
            windup_guard: 0.0_f32,
            proportional_gain: 0.0_f32,
            integral_gain: 0.0_f32,
            derivative_gain: 0.0_f32,
            prev_input: 0.0_f32,
            int_error: 0.0_f32,
            control: 0.0_f32,
            prev_steering_angle: 0.0_f32,
        }
    }

    pub fn zeroize(&mut self, integral_windup_guard: f32) {
        // set prev and integrated error to zero
        self.prev_input = 0.0_f32;
        self.int_error = 0.0_f32;
        self.prev_steering_angle = 0.0_f32;
        self.windup_guard = integral_windup_guard;
    }

    // TODO - proper result
    pub fn update(&mut self, setpoint: f32, input: f32, dt: f32) -> Option<()> {
        if dt <= 0.0_f32 {
            return None;
        }

        let curr_error = setpoint - input;

        // integration with windup guarding
        self.int_error += curr_error * dt;

        if self.int_error < -self.windup_guard {
            self.int_error = -self.windup_guard;
        } else if self.int_error > self.windup_guard {
            self.int_error = self.windup_guard;
        }

        // differentiation
        let diff = (input - self.prev_input) / dt;

        // scaling
        let p_term = self.proportional_gain * curr_error;
        let i_term = self.integral_gain * self.int_error;
        let d_term = self.derivative_gain * diff;

        // summation of terms
        self.control = p_term + i_term - d_term;

        // save current error as previous error for next iteration
        self.prev_input = input;

        Some(())
    }
}
