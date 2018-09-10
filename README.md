# oxcc

## Building

```bash
rustup default nightly

rustup component add rust-src
cargo install --force rustfmt-nightly

rustup target add thumbv7em-none-eabihf

cargo install svd2rust
```

```bash
cargo build
```

## Deploying

Install [stlink](https://github.com/texane/stlink) tools.

```bash
./scripts/deploy

# or manually
arm-none-eabi-objcopy \
    -O ihex \
    target/thumbv7em-none-eabihf/debug/oxcc \
    target/thumbv7em-none-eabihf/debug/oxcc.hex

st-flash --format ihex write target/thumbv7em-none-eabihf/debug/oxcc.hex
```

## Debugging

Install OpenOCD:

```bash
sudo apt-get install openocd
```

```bash
./scripts/run-openocd

# or manually
# openocd -f board/stm32f7discovery.cfg
```

```bash
cargo run

# or manually
# arm-none-eabi-gdb target/thumbv7em-none-eabihf/debug/oxcc
```

## Links

- [BSP crate](https://github.com/jonlamb-gh/nucleo-f767zi)
- [HAL crate](https://github.com/jonlamb-gh/stm32f767-hal)
- [device crate](https://github.com/adamgreig/stm32-rs/tree/master/stm32f7)
