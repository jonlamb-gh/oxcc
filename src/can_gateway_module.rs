// https://github.com/jonlamb-gh/oscc/tree/devel/firmware/can_gateway

use board::Board;

// TODO feature gate vehicles
use kial_soul_ev::*;

// TODO - use some form of println! logging that prefixes with a module name?

pub struct CanGatewayModule {}

impl CanGatewayModule {
    pub fn new() -> Self {
        CanGatewayModule {}
    }

    pub fn init_devices(&self, _board: &mut Board) {}

    // TODO - error handling
    pub fn republish_obd_frames_to_control_can_bus(&mut self, board: &mut Board) {
        if let Ok(rx_frame) = board.obd_can().receive() {
            let id: u32 = rx_frame.id().into();

            if (id == KIA_SOUL_OBD_STEERING_WHEEL_ANGLE_CAN_ID.into())
                || (id == KIA_SOUL_OBD_WHEEL_SPEED_CAN_ID.into())
                || (id == KIA_SOUL_OBD_BRAKE_PRESSURE_CAN_ID.into())
                || (id == KIA_SOUL_OBD_THROTTLE_PRESSURE_CAN_ID.into())
            {
                if let Err(_) = board.control_can().transmit(&rx_frame) {
                    // TODO - error handling
                }
            }
        }
    }
}
