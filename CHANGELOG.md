# rabbitmqadmin-ng Change Log

## v0.14.0 (Dec 22, 2024)

### Breaking Changes

 * Multi-word command line flags now use the more common `--snake-case[=]{value}` format
   instead of `rabbitmqadmin` v1's `lower_case={value}`.

### Enhancements

 * New command category: `health_check` which provides access to the [health check endpoints](https://www.rabbitmq.com/docs/monitoring#health-checks).

   Currently, only the three (arguably) most important health checks are implemented: `local_alarms`, `cluster_wide_alarms`, and `node_is_quorum_critical`.

   GitHub issues: [#33](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/33), [#34](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/34).

 * `health_check help` now includes a link to the respective RabbitMQ documentation guide. 


## v0.13.0 (Dec 21, 2024)

### Enhancements

 * Several key `delete` commands, namely `delete vhost`, `delete user`, `delete queue` and `delete exchange` now support a new flag, `--idempotently`.
   When this flag is used, 404 Not Found responses from the HTTP

   GitHub issue: [#32](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/32)

 * `--non-interactive` is a new global flag. When used, this flag will instruct `rabbitmqadmin` to not produce table border formatting,
    and generally try to use output that'd be easier to consume from scripts

 * Initial work on improving error reporting

## Bug Fixes

 * `declare exchange` propduced an incorrect API request payload
   when target exchange `--type` was an `x-*` type (a plugin provided-type), such as `x-local-random` or `x-consistent-hash`


## v0.12.0 (Dec 8, 2024)

### Enhancements

 * Implement support for configuration files. Instead of `.ini` files used by
   `rabbitmqadmin` v1, this version uses [TOML](https://toml.io/en/).
   
   GitHub issue: [#28](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/28)

 * Implement `show overview`

   GitHub issue: [#25](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/25)

 * `declare queue` no longer requires a `--queue-type`. If not type is specified,
   it will not be included into the request and the queue will be declared
   with the [default queue type of the target virtual host](https://www.rabbitmq.com/docs/vhosts#default-queue-type).

   GitHub issue: [#29](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/29)

### Releases

Release artifacts are no longer distributed as single file archives. Instead,
the release now includes "naked" binaries that can be downloaded and executed
without un-archiving.

GitHub issue: [#31](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/31)