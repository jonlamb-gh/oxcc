# OxCC Bootloader

An FOTA capable bootloader used to enable `OxCC` firmware
updates via TCP or CAN.

Inspired by [blethrs](https://github.com/AirborneEngineering/blethrs).

## Building

**NOTE**: debug builds take up too much space, the bootloader needs to fit in a specific flash sector.

Can this be detected at compile-time?

```bash
cargo build --release
```

## Deploying

```bash
./scripts/deploy
```

## Default Config

Without a valid config in flash, `OxCC` defaults to:

- IP address: `10.1.1.0/24`
- gateway: `10.1.1.1`
- MAC: `02:00:01:02:03:04`

## Debugging

```bash
cargo run --release
```

```text
|-=-=-=-=-=-=-= 0xCC Bootloader =-=-=-=-=-=-=-
| Version 0.1.0 7c0ed4b
| Platform thumbv7em-none-eabihf
| Built on Tue, 25 Sep 2018 13:20:52 GMT
| rustc 1.30.0-nightly (cb6d2dfa8 2018-09-16)
|-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

 Initialising cache...                OK
 Initialising clocks...               OK
 Initialising GPIOs...                OK
 Reading configuration...             OK
UserConfig:
  MAC Address: 02:00:03:07:03:05
  IP Address: 10.1.1.10/24
  Gateway: 10.1.1.1
  Checksum: B3819D1F

 Initialising Ethernet...             OK
 Waiting for link...                  OK
 Initialising network...              OK
 Ready.
```

## Using `firmware-updater`

An example host-side updater Python [script](firmware-updater) is used to talk to the bootloader.

```bash
./firmware-updator -h
```

### Info

```bash
./firmware-updater 10.1.1.10 info

Connecting to bootloader...
Received bootloader information:
Version: 0.1.0 c661da9
Built: Tue, 25 Sep 2018 15:52:39 GMT
Compiler: rustc 1.30.0-nightly (cb6d2dfa8 2018-09-16)
MCU ID: 303138353436511600450038
```

### Boot

```bash
./firmware-updater 10.1.1.10 boot

Connecting to bootloader...
Received bootloader information:
Version: 0.1.0 c661da9
Built: Tue, 25 Sep 2018 15:52:39 GMT
Compiler: rustc 1.30.0-nightly (cb6d2dfa8 2018-09-16)
MCU ID: 303138353436511600450038

Sending reboot command...
```

### Deploying New Firmware

#### Reset to Bootloader

Instruct OxCC firmware to reset into the bootloader with a CAN frame:

- CAN ID: `0xF0`
- DLC: `8`
- DATA: not used yet

Or hold down the user-button and reset the board.


#### Program Flash

```bash
# ELF to binary
arm-none-eabi-objcopy -O binary ../target/thumbv7em-none-eabihf/release/oxcc oxcc.bin

./firmware-updater 10.1.1.10 program oxcc.bin

Connecting to bootloader...
Received bootloader information:
Version: 0.1.0 c661da9
Built: Tue, 25 Sep 2018 15:52:39 GMT
Compiler: rustc 1.30.0-nightly (cb6d2dfa8 2018-09-16)
MCU ID: 303138353436511600450038

Erasing (may take a few seconds)...
Writing 53.14kB in 54 segments...
100%|██████████████████████████████████████████████████████████████████| 54/54 [00:00<00:00, 59.56kB/s]
Writing completed successfully. Reading back...
100%|██████████████████████████████████████████████████████████████████| 54/54 [00:00<00:00, 76.50kB/s]
Readback successful.
Sending reboot command...
```
