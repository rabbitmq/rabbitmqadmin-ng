# Contributing

## Running Tests

While tests support the standard `cargo test` option, another option
for running tests is [`cargo nextest`](https://nexte.st/).

### Test Structure

Tests are organized into three directories under `tests/`:

 * `tests/integration/`: integration tests that drive the CLI binary and require a running RabbitMQ node
 * `tests/unit/`: unit tests with no external dependencies
 * `tests/proptests/`: property-based tests

### Run All Tests

``` bash
NEXTEST_RETRIES=3 cargo nextest run --all-features
```

### Run Only Unit and Property-Based Tests (No Local RabbitMQ Node Needed)

``` bash
cargo nextest run -E 'binary(unit) or binary(proptests)'
```

### Run Only Integration Tests

``` bash
NEXTEST_RETRIES=3 cargo nextest run -E 'binary(integration)'
```

### Run a Specific Test

``` bash
NEXTEST_RETRIES=3 cargo nextest run -E "test(test_list_all_vhost_limits)"
```
