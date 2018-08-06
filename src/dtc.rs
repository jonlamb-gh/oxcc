pub trait DtcBitfield {
    fn set(&mut self, dtc: u8);
    fn clear(&mut self, dtc: u8);
    fn check(&self, dtc: u8) -> bool;
}

impl DtcBitfield for u8 {
    fn set(&mut self, dtc: u8) {
        *self |= 1 << dtc;
    }

    fn clear(&mut self, dtc: u8) {
        *self &= !(1 << dtc);
    }

    fn check(&self, dtc: u8) -> bool {
        *self & (1 << dtc) != 0
    }
}
