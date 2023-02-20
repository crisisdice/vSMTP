# Benchmarks

This directory contains multiple configuration to mesure the performances of vSMTP and compare them to other software.

## Client

* **stress**: Run vsmtp under heavy load with random tls settings, port and authentication coming from a client. Supports telemetry using the [opentelemetry crate](https://crates.io/crates/opentelemetry).

## Server

* **hold**: Compare the performances of vSMTP and Postfix when holding incoming messages.
* **dkim-dmarc**: Compare the performances of vSMTP and Postfix using dkim and dmarc.

It is recommended that you install the following postfix configurations using the `postmulti` command:

```sh
postmulti -I <name-of-instance> -G mta -e create
```

Each instance will have it's own configuration and set of queues.
