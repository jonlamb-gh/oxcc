//! Diagnostic trouble codes

pub trait DtcBitfield {
    fn set(&mut self, dtc: u8);
    fn clear(&mut self, dtc: u8);
    fn clear_all(&mut self);
    fn check(&self, dtc: u8) -> bool;
    fn are_any_set(&self) -> bool;
}

impl DtcBitfield for u8 {
    fn set(&mut self, dtc: u8) {
        *self |= 1 << dtc;
    }

    fn clear(&mut self, dtc: u8) {
        *self &= !(1 << dtc);
    }

    fn clear_all(&mut self) {
        *self = 0;
    }

    fn check(&self, dtc: u8) -> bool {
        *self & (1 << dtc) != 0
    }

    fn are_any_set(&self) -> bool {
        *self > 0
    }
}
