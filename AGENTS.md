# Instructions for AI Agents

## Overview

This is [`rabbitmqadmin v2`](https://www.rabbitmq.com/docs/management-cli), a CLI tool
that targets RabbitMQ's HTTP API.


## Build and Test

```bash
cargo build

cargo fmt --all

cargo nextest run --all-features
cargo clippy --all-features
```

To [filter](https://nexte.st/docs/filtersets/) tests with `cargo nextest`:

```bash
cargo nextest run -E "test(test_name)"
```

### Test Node Configuration

Test suites require a RabbitMQ node running on `localhost:15672` with `rabbitmq_management` plugin enabled.
`bin/ci/before_build.sh` is a script that demonstrates how the node should be configured.


## Key Files

### Implementation

 * `src/main.rs`: the entry point
 * `src/cli.rs`: `clap`-based CLI
 * `src/commands.rs`: command implementations
 * `src/config.rs`: configuration file support
 * `src/errors.rs`: error types
 * `src/output.rs`: table styling, output formatting
 * `src/tables.rs`: custom tables for certain commands
 * `src/tanzu_*.rs`: Tanzu RabbitMQ-specific commands

### Testing

 * `tests/fixtures/`: fixture files (e.g. definition files)
 * `tests/*_tests.rs`: test modules, require a running RabbitMQ nodes
 * `bin/ci/*`: CI and RabbitMQ node setup scripts


## Key Dependencies

 * `clap`: CLI framework
 * [`rabbitmq_http_client`](https://crates.io/crates/rabbitmq_http_client): RabbitMQ [HTTP API](https://www.rabbitmq.com/docs/http-api-reference) client
 * `serde`, `serde_json`: JSON serialization
 * `tabled` formats results as tables
 * Via `rabbitmq_http_client`: `reqwest` with `rustls` and `aws_lc_rs`: HTTP client, TLS and HTTPS

## Target Rust Version

 * This tool targets very recent Rust (such as `1.92`)

## Rust Code Style

 * Use top-level `use` statements (imports) to fully-qualified names, e.g. `Display` or `fmt::Display` with a `use` statement, to `std::fmt::Display`
 * Never use function-local `use` statements (imports)
 * Add tests to the modules under `tests`, never in the implementation files
 * At the end of each task, run `cargo fmt --all`
 * At the end of each task, run `cargo clippy --all` and fix any warnings it might emit

## Comments

 * Only add very important comments, both in tests and in the implementation

## Git Instructions

 * Never add yourself to the list of commit co-authors

## Style Guide

 * Never add full stops to Markdown list items
