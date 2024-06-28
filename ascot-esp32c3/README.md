# Ascot firmware for an `ESP32-C3` board

An `Ascot`-compliant HTTP server, mainly thought as a **IoT** firmware,
for an `ESP32-C3` board.

This firmware also implements a thin-layer library to manage routes, errors and
hazards.

## Prerequisites

Follow the [Prerequisites](https://github.com/esp-rs/esp-idf-template#prerequisites) 
section in the `esp-idf-template` crate.

Specific prerequisites for the project:

- Connect the board to a system through its serial connection
- Fill in the `wifi_confi.toml` with the Wi-Fi credentials: `SSID` and `PASSWORD`

## Building

To build the code in `debug` mode:

```console
cargo build
```

To build the code in `release` mode in order to have all optimizations enabled:

```console
cargo build --release
```

## Running

To flash and run the firmware on an `ESP32-C3` board:

```console
cargo run [--release]
```

The optional `--release` parameter enables all optimizations and makes the
final firmware smaller.

## Board usage on WSL

Support for connecting `USB` devices is not natively available on [Windows
Subsystem for Linux (WSL)](https://learn.microsoft.com/en-us/windows/wsl/).

In order to use the `ESP32-C3` board with `WSL`, follow this
[guide](https://learn.microsoft.com/en-us/windows/wsl/connect-usb) and manually
connect the `USB` port used by the board to `WSL`.
