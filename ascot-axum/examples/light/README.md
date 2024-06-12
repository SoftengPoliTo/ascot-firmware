# Light

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
curl -X PUT 127.0.0.1:3000/light/on/4.0/true
```

```console
curl -X POST 127.0.0.1:3000/light/on/4.0/true
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

```
Starting the Ascot server at 0.0.0.0:3000
```
