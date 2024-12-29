# rabbitmqadmin v2: a Modern Command Line Client for the [RabbitMQ HTTP API](https://www.rabbitmq.com/docs/management#http-api)

`rabbitmqadmin` v2 is a major revision of `rabbitmqadmin`, one of the [RabbitMQ CLI tools](https://www.rabbitmq.com/docs/cli)
that target the [HTTP API](https://www.rabbitmq.com/docs/management#http-api).

If you are migrating from the original `rabbitqadmin`, please see [Breaking or Potentially Breaking Changes](#breaking-or-potentially-breaking-changes)
to learn about a few breaking change in the interface.

## Getting Started

### Installation

#### Binary Releases

To download a binary build, see [Releases](https://github.com/rabbitmq/rabbitmqadmin-ng/releases).

#### Building from Source with `cargo install`

On platforms not covered by the binary builds, `rabbitmqadmin` v2 can be installed with [Cargo](https://doc.rust-lang.org/cargo/commands/cargo-install.html):

```shell
cargo install rabbitmqadmin
```

### Documentation

For usage documentation, see [Usage](#usage).

## Project Maturity

This version of `rabbitmqadmin` should be considered reasonably mature to be used.

Before migrating, please see [Breaking or Potentially Breaking Changes](#breaking-or-potentially-breaking-changes) to learn about a few breaking change in the interface.

### Known Limitations

The following `rabbitmqadmin` v1 features are not currently implemented:

* [Configuration file support](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/28)
* Support for TLS client (x.509, HTTPS) [certificate and private key](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/26)


## Usage

### Exploring Available Command Groups and Sub-commands

To explore what command groups are available, use

```shell
rabbitmqadmin help
```

which will output a list of command groups:

```
Usage: rabbitmqadmin [OPTIONS] <command>

Commands:
  show                 overview
  list                 lists objects by type
  declare              creates or declares things
  delete               deletes objects
  purge                purges queues
  health_check         runs health checks
  close                closes connections
  rebalance            rebalances queue leaders
  definitions          operations on definitions
  export               see 'definitions export'
  import               see 'definitions import'
  feature_flags        operations on feature flags
  deprecated_features  operations on deprecated features
  publish              Publishes (inefficiently) message(s) to a queue or a stream. Only suitable for development and test environments.
  get                  fetches message(s) from a queue or stream via polling. Only suitable for development and test environments.
  help                 Print this message or the help of the given subcommand(s)
```

To explore commands in a specific group, use

```shell
rabbitmqadmin {group name} help
```

### Exploring the CLI with `help`, `--help`

To learn about what command groups and specific commands are available, run

``` shell
rabbitmqadmin help
```

This flag can be appended to a command or subcommand to get command-specific documentation:

```shell
rabbitmqadmin declare queue --help
# => creates or declares things
# =>
# => Usage: rabbitmqadmin declare [object]
# => ...
```

Alternatively, the `help` subcommand can be given a command name. It's the equivalent
of tagging on `--help` at the end of command name:

```shell
rabbitmqadmin declare help queue
# => creates or declares things
# =>
# => Usage: rabbitmqadmin declare [object]
# => ...
```

More specific examples are covered in the Examples section below.


### Interactive vs. Use in Scripts

Like the original version, `rabbitmqadmin` v2 is first and foremost built for interactive use
by humans. Many commands will output formatted tables, for example:

```shell
rabbitmqadmin show overview
```

will output a table that looks like this:

```
┌──────────────────┬───────────────────────────────────────────────────────────────────────────────────────────────────┐
│ Overview                                                                                                             │
├──────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────┤
│ key              │ value                                                                                             │
├──────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Product name     │ RabbitMQ                                                                                          │
├──────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Product version  │ 4.0.5                                                                                             │
├──────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────┤
│ RabbitMQ version │ 4.0.5                                                                                             │
├──────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Erlang version   │ 26.2.5.6                                                                                          │
├──────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Erlang details   │ Erlang/OTP 26 [erts-14.2.5.5] [source] [64-bit] [smp:10:10] [ds:10:10:10] [async-threads:1] [jit] │
└──────────────────┴───────────────────────────────────────────────────────────────────────────────────────────────────┘
```

As it is easy to observe, parsing such output in a script will be challenging.

For this reason, `rabbitmqadmin` v2 can render results in a way that would be much more friendly
for scripting if the `--non-interactive` flag is passed. It is a global flag so it must be
passed before the command and subcommand name:

```shell
rabbitmqadmin --non-interactive show overview
```

The output of the above command will not include any table borders and will is much easier to parse
as a result:

```
 key
 Product name      RabbitMQ
 Product version   4.0.5
 RabbitMQ version  4.0.5
 Erlang version    26.2.5.6
 Erlang details    Erlang/OTP 26 [erts-14.2.5.5] [source] [64-bit] [smp:10:10] [ds:10:10:10] [async-threads:1] [jit]
```

### Retrieving Basic Node Information

``` shell
rabbitmqadmin show overview
```

will display essential node information in tabular form.

### Retrieving Connection, Queue/Stream, Channel Churn Information

Helps assess connection, queue/stream, channel [churn metrics](https://rabbitmq.com/docs/connections#high-connection-churn) in the cluster.

``` shell
rabbitmqadmin show churn
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

### Create a Virtual Host

```shell
rabbitmqadmin declare vhost --name "vh-789" --default-queue-type "quorum" --description "Used to reproduce issue #789"
```

### Delete a Virtual Host

```shell
rabbitmqadmin delete vhost --name "vh-789"
```

```shell
# --idempotently means that 404 Not Found responses will not be  considered errors
rabbitmqadmin delete vhost --name "vh-789" --idempotently
```


### Declare a Queue

```shell
rabbitmqadmin --vhost "events" declare queue --name "target.quorum.queue.name" --type "quorum" --durable true
```

```shell
rabbitmqadmin --vhost "events" declare queue --name "target.stream.name" --type "stream" --durable true
```

```shell
rabbitmqadmin --vhost "events" declare queue --name "target.classic.queue.name" --type "classic" --durable false --auto-delete true
```

### Delete a queue

``` shell
rabbitmqadmin --vhost "events" delete queue --name "target.queue.name"
```

``` shell
# --idempotently means that 404 Not Found responses will not be  considered errors
rabbitmqadmin --vhost "events" delete queue --name "target.queue.name" --idempotently
```

### Declare an Exchange

```shell
rabbitmqadmin --vhost "events" declare exchange --name "events.all_types.topic" --type "topic" --durable true
```

```shell
rabbitmqadmin --vhost "events" declare exchange --name "events.all_type.uncategorized" --type "fanout" --durable true --auto-delete false
```

```shell
rabbitmqadmin --vhost "events" declare exchange --name "local.random.c60bda92" --type "x-local-random" --durable true
```

### Delete an exchange

``` shell
rabbitmqadmin --vhost "events" delete exchange --name "target.exchange.name"
```

``` shell
# --idempotently means that 404 Not Found responses will not be  considered errors
rabbitmqadmin --vhost "events" delete exchange --name "target.exchange.name" --idempotently
```

### List feature flags and their state

```shell
rabbitmqadmin feature_flags list
```

```shell
# same command as above
rabbitmqadmin list feature_flags
```

### Enable a feature flag

```shell
rabbitmqadmin feature_flags enable rabbitmq_4.0.0
```

### Enable all stable feature flags

```shell
rabbitmqadmin feature_flags enable_all
```

### List deprecated features in use in the cluster

```shell
rabbitmqadmin deprecated_features list_used
```

### List all deprecated features

```shell
rabbitmqadmin deprecated_features list
```

```shell
# same command as above
rabbitmqadmin list deprecated_features
```


## Configuration Files

`rabbitmqadmin` v2 supports [TOML](https://toml.io/en/)-based configuration files
stores groups of HTTP API connection settings under aliases ("node names" in original `rabbitmqadmin` speak). 

Here is an example `rabbitmqadmin` v2 configuration file:

```toml
[local]
hostname = "localhost"
port = 15672
username = "lolz"
password = "lolz"
vhost = '/'

[staging]
hostname = "192.168.20.31"
port = 15672
username = "staging-2387a72329"
password = "staging-1d20cfbd9d"

[production]
hostname = "(redacted)"
port = 15671
username = "user-2ca6bae15ff6b79e92"
password = "user-92ee4c479ae604cc72"
```

Instead of specifying `--hostname` or `--username` on the command line to connect to
a cluster (or specific node) called `staging`, a `--node` alias can be specified instead:

```shell
# will use the settings from the section called [staging]
rabbitmqadmin --node staging show churn
```

Default configuration file path is at `$HOME/.rabbitmqadmin.conf`, as it was in
the original version of `rabbitmqadmin`. It can be overridden on the command line:

```shell
# will use the settings from the section called [staging]
rabbitmqadmin --config $HOME/.configuration/rabbitmqadmin.conf --node staging show churn
```


## Project Goals Compared to `rabbitmqadmin` v1

This version of `rabbitmqadmin` has a few ideas in mind:

* This is a major version bump. Therefore, reasonable breaking changes are OK. `rabbitmqadmin` hasn't seen a revision in fourteen years
* `rabbitmqadmin` should be standalone binary. There are very few reasons not to build and distribute it that way
* Standalone project, not an obscure feature: `rabbitmqadmin` should be a standalone tool, not a relatively unknown "feature" of
  the RabbitMQ management plugin, and should be developed as such, not tied completely to the development
  environment, practices and release schedule of RabbitMQ itself
* v2 should be a distributed via GitHub releases and not a special `rabbitmq_management` endpoint
* There is a lot of room to improve validation of flags and arguments, since breaking changes are OK for v2
* This tool should strive to be as free as practically possible from CVEs in other projects that show up on security scans.
  CVEs from older Python versions should not plague OCI images that choose to include `rabbitmqadmin` v2


## Breaking or Potentially Breaking Changes

### --snake-case for Command Options

`rabbitmqadmin` v1 used `lower_case` for named command arguments, for example:

```shell
# Note: auto_delete
rabbitmqadmin-v1 --vhost "vh-2" declare queue name="qq.1" type="quorum" durable=true auto_delete=false
```

`rabbitmqadmin` v2 uses a more typical `--snake-case` format for the same arguments:

```shell
# Note: --auto-delete
rabbitmqadmin --vhost "vh-2" declare queue --name "qq.1" --type "quorum" --durable true --auto-delete false 
```

### Global Arguments Come First

Global flags in `rabbitmqadmin` v2 must precede the command category (e.g. `list`) and the command itself,
namely various HTTP API endpoint options and `--vhost`:

```shell
rabbitmqadmin --vhost "events" declare queue --name "target.quorum.queue.name" --type "quorum" --durable true
```

### --prefix Overrides API Path Prefix

In `rabbitmqadmin` v1, `--path-prefix` appended to the default [API path prefix](https://rabbitmq.com/docs/management#path-prefix).
In this version, the value passed to `--path-prefix` will be used as given, in other words,
it replaces the default prefix, `/api`.

### Configuration File Format Moved to TOML

`rabbitmqadmin` v1 supported ini configuration files that allowed
the user to group a number of command line values under a name, e.g. a cluster or node nickname.

Due to the "no dependencies other than Python" design goal of `rabbitmqadmin` v1, this feature was not really tested,
and the specific syntax (that of ini files, supported by Python's [`ConfigParser`](https://docs.python.org/3/library/configparser.html)) linting, parsing or generation tools were not really available.

`rabbitmqadmin` v2 replaces this format with [TOML](https://toml.io/en/), a popular configuration standard
with [verification and linting tools](https://www.toml-lint.com/), as well as very mature parser
that is not at all specific to `rabbitmqadmin` v2.

Here is an example `rabbitmqadmin` v2 configuration file:

```toml
[local]
hostname = "localhost"
port = 15672
username = "lolz"
password = "lolz"
vhost = '/'

[staging]
hostname = "192.168.20.31"
port = 15672
username = "staging-2387a72329"
password = "staging-1d20cfbd9d"

[production]
hostname = "(redacted)"
port = 15671
username = "user-efe1f4d763f6"
password = "(redacted)"
```


## Getting Help

Please use GitHub Discussions in this repository and [RabbitMQ community Discord server](https://rabbitmq.com/discord/).


## License

This tool, `rabbitmqadmin` (v2 and later versions), is dual-licensed under
the Apache Software License 2.0 and the MIT license.
