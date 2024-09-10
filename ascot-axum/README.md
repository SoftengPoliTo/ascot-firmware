# ascot-axum

This library crate provides a series of APIs to build an `axum` server which
represents the firmware of an IoT device.

Among its functionalities, it can interact send and receive data through
REST APIs.

The implemented devices can be found inside the [examples](./examples)
directory.

# Statically-linked firmware device

In order to build a statically-linked firmware device,
run the following command:

```bash
cargo build --package firmware_device --target=x86_64-unknown-linux-musl
```

where `firmware_device` is the name of the example you want to build.


