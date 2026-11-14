# conn-test

[![Rust](https://github.com/DoumanAsh/conn-test/actions/workflows/rust.yml/badge.svg)](https://github.com/DoumanAsh/conn-test/actions/workflows/rust.yml)
[![Hub](https://img.shields.io/badge/Docker-Hub-2496ed.svg)](https://hub.docker.com/r/douman/conn-test/tags)
[![Quay](https://img.shields.io/badge/quay.io-douman%2Fconn--test-blue)](https://quay.io/repository/doumanash/conn-test)

Simple utility to test connectivity

## Usage

```
conn-test 0.1.0
Connectivity testing utility

USAGE: [OPTIONS]

OPTIONS:
    -h,  --help                           Prints this help information
    -q,  --quiet                          Specifies whether to silence verbose logging of errors. Can be set via env CONN_TEST_QUIET
         --tcp_timeout <tcp_timeout>      Specifies timeout in seconds for TCP connections. Defaults to 5s. Can be set via env CONN_TEST_TCP_TIMEOUT
         --tcp <tcp>...                   Hostname alongside port to check by simple TCP connection. Can be init via env CONN_TEST_TCP with comma separator
         --kafka_timeout <kafka_timeout>  Specifies timeout in seconds for Kafka connections. Defaults to 5s. Can be set via env CONN_TEST_KAFKA_TIMEOUT
         --kafka <kafka>...               Bootstrap broker to check by performing connection over plain SASL. Credentials can be supplied in URL. Can be init via env CONN_TEST_KAFKA with comma separator
         --mongo_timeout <mongo_timeout>  Specifies timeout in seconds for Mongo connections. Defaults to 5s. Can be set via env CONN_TEST_MONGO_TIMEOUT
         --mongo <mongo>...               MongoDB connection URLs. Credentials can be supplied in URL. Can be init via env CONN_TEST_MONGO with comma separator
```
