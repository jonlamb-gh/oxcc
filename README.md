# OxCC

## Overview

A port of [Open Source Car Control](https://github.com/jonlamb-gh/oscc) written in Rust.

`OxCC` runs on the [NUCLEO-F767ZI STM32F767ZI](https://www.st.com/en/evaluation-tools/nucleo-f767zi.html) board.

It is built around the traits and patterns provided by the [embedded-hal](https://github.com/rust-embedded/embedded-hal)
project and community:
see the [BSP crate](https://github.com/jonlamb-gh/nucleo-f767zi),
the [HAL crate](https://github.com/jonlamb-gh/stm32f767-hal),
and the [device crate](https://github.com/adamgreig/stm32-rs/tree/master/stm32f7).

### OSCC Divergence

Apart from the change in MCU/board, `OxCC` combines all of the OSCC modules (throttle, brake, steering, CAN gateway) into a single application.

#### Hardware

* CAN

  `OxCC` uses the stm's on-board bxCAN controller.
  For a transceiver I've been using the [SN65HVD230](https://www.waveshare.com/sn65hvd230-can-board.htm) from Waveshare.

## Getting Started

### Dependencies

* [rust](https://github.com/rust-lang-nursery/rustup.rs) (nightly)
* [svd2rust](https://github.com/rust-embedded/svd2rust)
* [openocd](http://openocd.org/) (for debugging)
* [gdb-arm-none-eabi](https://gcc.gnu.org/) (for debugging)
* [binutils-arm-none-eabi](https://gcc.gnu.org/) (uses `objcopy` for device deployment)
* [stlink](https://github.com/texane/stlink) (for device deployment)

### Building

The default Cargo configuration will build for the `Kia Soul EV` vehicle
with the `panic-over-abort` strategy.

See the `[features]` section of the [Cargo.toml](Cargo.toml) to change configurations.

* Install system package dependencies:
  ```bash
  sudo apt-get install openocd
  sudo apt-get install gdb-arm-none-eabi
  sudo apt-get install binutils-arm-none-eabi
  ```
* Install `stlink` from source: [guide](https://github.com/texane/stlink/blob/master/doc/compiling.md)
* Install Rust nightly and additional components:
  ```bash
  curl https://sh.rustup.rs -sSf | sh
  rustup install nightly
  rustup component add rust-src
  rustup component add rustfmt-preview --toolchain nightly
  rustup target add thumbv7em-none-eabihf
  ```
* Install `svd2rust`:
  ```bash
  cargo install svd2rust
  ```
* Build `OxCC` firmware:
  ```bash
  cargo build
  ```

### Deploying

Deploy the firmware Using `st-flash` (provided by `stlink`):

```bash
# Convert ELF to ihex format
arm-none-eabi-objcopy \
    -O ihex \
    target/thumbv7em-none-eabihf/release/oxcc \
    target/thumbv7em-none-eabihf/release/oxcc.hex

# Upload to flash
st-flash \
    --format ihex \
    write \
    target/thumbv7em-none-eabihf/release/oxcc.hex
```

## Debugging

In one terminal, start `openocd`:

```bash
openocd -f board/stm32f7discovery.cfg
```

In the `OxCC` terminal, use the [runner](.cargo/config) to start a `gdb` session:

```bash
cargo run
```

# License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
