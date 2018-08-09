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
    }
}
