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

### Run benchmarks

```
# Size (MB)	0.1
# Msg	100000
./run.sh 4 100000 100000
./run.sh 8 100000 100000
./run.sh 12 100000 100000
./run.sh 16 100000 100000
./run.sh 32 100000 100000

# Size (MB)	0.5
# Msg	20000
./run.sh 4 500000 20000
./run.sh 8 500000 20000
./run.sh 12 500000 20000
./run.sh 16 500000 20000
./run.sh 32 500000 20000

# Size (MB)	1
# Msg	10000
./run.sh 4 1000000 10000
./run.sh 8 1000000 10000
./run.sh 12 1000000 10000
./run.sh 16 1000000 10000
./run.sh 32 1000000 10000

# Size (MB)	5
# Msg	2000
./run.sh 4 5000000 2000
./run.sh 8 5000000 2000
./run.sh 12 5000000 2000
./run.sh 16 5000000 2000
./run.sh 32 5000000 2000

# Size (MB)	10
# Msg	1000
./run.sh 4 10000000 1000
./run.sh 8 10000000 1000
./run.sh 12 10000000 1000
./run.sh 16 10000000 1000
./run.sh 32 10000000 1000
```
