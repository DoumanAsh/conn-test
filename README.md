# conn-test

[![Rust](https://github.com/DoumanAsh/conn-test/actions/workflows/rust.yml/badge.svg)](https://github.com/DoumanAsh/conn-test/actions/workflows/rust.yml)

Simple utility to test connectivity

## Usage

```
Connectivity testing utility

USAGE: [OPTIONS]

OPTIONS:
    -h,  --help                       Prints this help information
    -q,  --quiet                      Specifies whether to silence verbose logging of errors. Can be set via env CONN_TEST_QUIET
         --tcp_timeout <tcp_timeout>  Specifies timeout in seconds for TCP connections. Defaults to 5s. Can be set via env CONN_TEST_TCP_TIMEOUT
         --tcp <tcp>...               Hostname alongside port to check by simple TCP connection. Can be init via env CONN_TEST_TCP with comma separator
```
