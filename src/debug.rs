// this will be replaced with debug console

use cortex_m;
use nucleo_f767zi::hal::prelude::*;
use nucleo_f767zi::hal::serial::Tx;
use nucleo_f767zi::hal::stm32f7x7::USART3;

pub struct DebugOutputHandle<'a> {
    tx: &'a mut Tx<USART3>,
}

impl<'a> DebugOutputHandle<'a> {
    pub fn init(tx: &'a mut Tx<USART3>) -> Self {
        DebugOutputHandle { tx }
    }
}

impl<'p> ::core::fmt::Write for DebugOutputHandle<'p> {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for &b in s.as_bytes() {
            block!(self.tx.write(b as _)).ok();
        }
        Ok(())
    }
}
