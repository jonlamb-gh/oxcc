// TODO
// https://github.com/jonlamb-gh/oscc/tree/devel/firmware/brake/kia_soul_petrol

use board::Board;
use nucleo_f767zi::hal::can::CanFrame;
use nucleo_f767zi::hal::prelude::*;
use super::types::*;

pub struct BrakeModule {}

impl BrakeModule {
    pub fn new(brake_dac:BrakeDac, brake_pins:BrakePins) -> Self {
        panic!("TODO - Kia Soul Petrol brake module not implemented yet");
        BrakeModule {}
    }

    pub fn init_devices(&self) {
        board.brake_spoof_enable().set_low();
        board.brake_light_enable().set_low();
        // TODO - PIN_DAC_CHIP_SELECT, HIGH
    }

    pub fn check_for_faults(&mut self, _board: &mut Board) {}

    pub fn publish_brake_report(&mut self, _board: &mut Board) {}

    pub fn process_rx_frame(&mut self, _can_frame: &CanFrame, _board: &mut Board) {}
}
