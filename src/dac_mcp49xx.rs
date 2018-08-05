// TODO - this will be replaced by a device driver crate DAC_MCP49xx
// https://github.com/jonlamb-gh/oscc/blob/master/firmware/common/libs/dac/oscc_dac.cpp

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
    pub fn prevent_signal_discontinuity(&mut self, output_a: u16, output_b: u16) {
        self.set_outputs(output_a, output_b);
    }
}
