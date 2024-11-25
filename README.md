# rabbitmqadmin v2

`rabbitmqadmin` v2 is a major revision of one of the RabbitMQ's CLI tools.

If you are migrating from the original `rabbitqadmin`, please see [CLI Changes](#cli-changes)
to learn about a few breaking change in the interface.

To download a binary build, see [Releases](https://github.com/rabbitmq/rabbitmqadmin-ng/releases).

For usage documentation, see [Usage](#usage).


## Project Goals

This version of `rabbitmqadmin` has a few ideas in mind:

 * This is a major version bump. Therefore, breaking changes are OK. `rabbitmqadmin` hasn't seen a revision in thirteen years
 * `rabbitmqadmin` should be standalone binary. There are very few reasons not to build and distribute it that way
 * v2 should be a distributed via GitHub releases and not a special `rabbitmq_management` endpoint
 * There is a lot of room to improve validation of flags and arguments, since breaking changes are OK for v2
 * Output should be revisited: what columns are output by default, whether columns should be selectable
 * Support for JSON and CSV was a popular addition in `rabbitmqctl`, `rabbitmq-diagnostics`, etc. Perhaps `rabbitmqadmin` should consider supporting them, too?


## Project Maturity

This version of `rabbitmqadmin` should be considered reasonably mature to be used.

Before migrating, please see [CLI Changes](#cli-changes) to learn about a few breaking change in the interface.

### Known Limitations

The following `rabbitmqadmin` v1 features are not currently implemented:

* [Configuration file support](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/28)
* Support for TLS client (x.509, HTTPS) [certificate and private key](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/26)


## Usage

### Getting Help

To learn about what command groups and specific commands are available, run

``` shell
rabbitmqadmin --help
```

Note that in this version, **global flags must precede the command category (e.g. `list`) and the command itself**:

```shell
rabbitmqadmin --vhost "events" declare queue --name "target.quorum.queue.name" --type "quorum" --durable true
```

The same command will display global flags. To learn about a specific command, append
`--help` to it:

``` shell
rabbitmqadmin declare queue --help
```

### Retrieving Basic Node Information

``` shell
rabbitmqadmin show overview
```

### Retrieving Connection, Queue/Stream, Channel Churn Information

Helps assess connection, queue/stream, channel [churn metrics](https://www.rabbitmq.com/docs/connections#high-connection-churn) in the cluster.

``` shell
rabbitmqadmin show overview
```

### Listing cluster nodes

``` shell
rabbitmqadmin list nodes
```

### Listing virtual hosts

``` shell
rabbitmqadmin list vhosts
```

### Listing users

``` shell
rabbitmqadmin list users
```

### Listing queues

``` shell
rabbitmqadmin list queues
```

``` shell
rabbitmqadmin --vhost "monitoring" list queues
```

### Listing exchanges

``` shell
rabbitmqadmin list exchanges
```

``` shell
rabbitmqadmin --vhost "events" list exchanges
```

### Listing bindings

``` shell
rabbitmqadmin list bindings
```

``` shell
rabbitmqadmin --vhost "events" list bindings
```

### Declare a queue

```shell
rabbitmqadmin --vhost "events" declare queue --name "target.quorum.queue.name" --type "quorum" --durable true
```

```shell
rabbitmqadmin --vhost "events" declare queue --name "target.stream.name" --type "stream" --durable true
```

```shell
rabbitmqadmin --vhost "events" declare queue --name "target.classic.queue.name" --type "classic" --durable false --auto_delete true
```

### Delete a queue

``` shell
rabbitmqadmin --vhost "events" delete queue --name "target.queue.name"
```

## Differences from `rabbitmqadmin` v1

Compared to the original `rabbitmqadmin`, this version:

 * Is distributed as a standalone binary and does not depend on Python
 * Uses much stricter CLI argument validation and has (relatively minor) breaking changes in the CLI
 * Is better documented
 * May choose to use different defaults where it makes sense

### CLI Changes

#### Global Arguments Come First

Global flags in `rabbitmqadmin` v2 must precede the command category (e.g. `list`) and the command itself,
namely various HTTP API endpoint options and `--vhost`:

```shell
rabbitmqadmin --vhost "events" declare queue --name "target.quorum.queue.name" --type "quorum" --durable true
```

### Getting Help

Please use GitHub Discussions in this repository and [RabbitMQ community Discord server](https://rabbitmq.com/discord/).


## License

This tool, `rabbitmqadmin` (v2 and later versions), is dual-licensed under
the Apache Software License 2.0 and the MIT license.
