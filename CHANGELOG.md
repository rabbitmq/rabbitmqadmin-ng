# rabbitmqadmin-ng Change Log

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