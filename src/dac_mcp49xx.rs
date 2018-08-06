// TODO - this will be replaced by a device driver crate DAC_MCP49xx
// https://github.com/jonlamb-gh/oscc/blob/master/firmware/common/libs/dac/oscc_dac.cpp

use dual_signal::DualSignal;

pub struct Mcp49xx {
    // TODO
}

impl Mcp49xx {
    pub const fn new() -> Self {
        Mcp49xx {}
    }

    pub fn set_outputs(&mut self, _output_a: u16, _output_b: u16) {
        // TODO
        // cortex_m::interrupt::free(|cs| { ... });
    }

    // TODO - not sure we'll need this, since the STM can do a lot in hw
    pub fn prevent_signal_discontinuity(&mut self, signal: &DualSignal) {
        self.set_outputs(signal.dac_output_a(), signal.dac_output_b());
    }
}
