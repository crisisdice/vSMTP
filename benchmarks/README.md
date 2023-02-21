# Benchmarks

This directory contains multiple configuration to mesure the performances of vSMTP and compare them to other software.

## Client

* **stress**: Run vsmtp under heavy load with random tls settings, port and authentication coming from a client. Supports telemetry using the [opentelemetry crate](https://crates.io/crates/opentelemetry).

## Server

* **hold**: Compare the performances of vSMTP and Postfix when holding incoming messages.
* **dkim-dmarc**: Compare the performances of vSMTP and Postfix using dkim and dmarc.

### Install

Install all benchmarks using the `install.sh` script. This scripts creates multiple postfix configurations using [`postmulti`](https://www.postfix.org/MULTI_INSTANCE_README.html), and multiple vsmtp configurations by storing them in `/etc/vsmtp/benchmarks/<bench-name>`.

Each vsmtp/postfix instance will have it's own configuration and set of queues.

### Using docker

It is possible to run all benchmarks inside a docker container if to prevent installing all dependencies on the system. (Though it might impact the results of the benchmarks)

```sh
docker run \
    -v .:/benchmarks \
    -it ubuntu:latest bash
```

Once the container has started and the shell is up, use the following commands to install the benchmarks.

```sh
cd benchmarks
./install.sh
```
