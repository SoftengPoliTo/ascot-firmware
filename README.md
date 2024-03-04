# Ascot Firmware

The `Ascot Firmware` is a REST server developed with the `Ascot` interface,
which provides a series of abstractions to interact with the commands defined
on a smart home device.
Through `REST` APIs, the server can run tasks on a device, retrieve some
of its data, or change some of its properties. The device implemented in this
example is a light.

The light is discoverable within the network through a `mDNS-SD` service.

## Building

Run the command

```console
cargo build
```

## Running the server

The server runs on `localhost` and listens to port `3000`. To make it run:

```console
cargo run
```

## REST API examples

Through `curl` or a web browser, it is possible to call the APIs which perform
some actions on a device.

**Turn a light on**

```console
curl -X PUT 127.0.0.1:3000/light/on
```

**Turn a light off**

```console
curl -X PUT 127.0.0.1:3000/light/off
```

**Toggle a light**

```console
curl -X PUT 127.0.0.1:3000/light/toggle
```

At the server startup, an initial message signalling its effective execution and
port number is printed.

Before an action is performed on a device, a message with the REST API
kind which triggers it is printed.

```
Starting the Ascot server (port 3000)…
Performing the REST API (PUT)…
```
