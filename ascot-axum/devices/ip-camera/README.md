# Ip-Camera Server

An Ip-Camera server usable from different operating systems for sending images
of the surrounding environment.

# Building

To build the camera run the command:

```console
cargo build
```

## Cross-compilation to aarch64 (ARM64) architecture

Install a binary named [cross](https://github.com/cross-rs/cross) which allow
to easily cross-compile Rust projects using Docker, without messing with
custom `Dockerfile`s.

```console
cargo install -f cross
```

To build the binary for `ARM64` architecture run:

```console
cross build [--release] --target=aarch64-unknown-linux-musl
```

where `--release` is an optional argument which enables 
all time and memory optimizations.
