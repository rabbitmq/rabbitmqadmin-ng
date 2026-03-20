# Contributing

## Running Tests

Most tests require a locally running RabbitMQ node. The easiest way to get one is via Docker.

### Prerequisites

Install [cargo-nextest](https://nexte.st/) if you don't have it:

```bash
cargo install cargo-nextest
```

### Step 1: Start RabbitMQ

```bash
docker run -d --name rabbitmq \
  -p 15672:15672 \
  -p 5672:5672 \
  rabbitmq:4.1-management
```

Wait for the node to boot:

```bash
sleep 15
```

### Step 2: Pre-configure the Node

Run the setup script using the Docker exec variant of `rabbitmqctl`:

```bash
RUST_HTTP_API_CLIENT_RABBITMQCTL=DOCKER:rabbitmq bin/ci/before_build.sh
```

This enables the required plugins (management, shovel, federation, stream), creates test users,
sets up the `rust/rabbitmqadmin` vhost, sets the cluster name, and enables all feature flags.

Wait for the changes to apply:

```bash
sleep 10
```

### Step 3: Run All Tests

```bash
NEXTEST_RETRIES=3 cargo nextest run --all-features
```

`NEXTEST_RETRIES=3` retries each failing test up to 3 times. This is recommended because some
tests depend on management plugin stats that can lag slightly behind the actual broker state.

### Stopping the Node

```bash
docker stop rabbitmq && docker rm rabbitmq
```

---

## Test Structure

Tests are organized into three directories under `tests/`:

 * `tests/integration/`: integration tests that drive the CLI binary and require a running RabbitMQ node
 * `tests/unit/`: unit tests with no external dependencies
 * `tests/proptests/`: property-based tests

### Run Only Unit and Property-Based Tests (No Local RabbitMQ Node Needed)

```bash
cargo nextest run -E 'binary(unit) or binary(proptests)'
```

### Run Only Integration Tests

```bash
NEXTEST_RETRIES=3 cargo nextest run -E 'binary(integration)'
```

### Run a Specific Test

```bash
NEXTEST_RETRIES=3 cargo nextest run -E "test(test_list_all_vhost_limits)"
```
