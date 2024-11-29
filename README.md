# rabbitmqadmin v2

`rabbitmqadmin` v2 is a major revision of one of the RabbitMQ's CLI tools.

If you are migrating from the original `rabbitqadmin`, please see [Breaking or Potentially Breaking Changes](#changes-compared-to-v1)
to learn about a few breaking change in the interface.

To download a binary build, see [Releases](https://github.com/rabbitmq/rabbitmqadmin-ng/releases).

For usage documentation, see [Usage](#usage).


## Project Goals Compared to `rabbitmqadmin` v1

This version of `rabbitmqadmin` has a few ideas in mind:

 * This is a major version bump. Therefore, reasonable breaking changes are OK. `rabbitmqadmin` hasn't seen a revision in fourteen years
 * `rabbitmqadmin` should be standalone binary. There are very few reasons not to build and distribute it that way
* Standalone project, not an obscure feature: `rabbitmqadmin` should be a standalone tool, not a relatively unknown "feature" of
  the RabbitMQ management plugin, and should be developed as such, not tied completely to the development
  environment, practices and release schedule of RabbitMQ itself
 * v2 should be a distributed via GitHub releases and not a special `rabbitmq_management` endpoint
 * There is a lot of room to improve validation of flags and arguments, since breaking changes are OK for v2
 * This tool as free as practically possible from CVEs in other projects that show up on security scans.
   CVEs from older Python versions should not plague OCI images that choose to include `rabbitmqadmin`
 * Output should be revisited: what columns are output by default, whether columns should be selectable

## Project Maturity

This version of `rabbitmqadmin` should be considered reasonably mature to be used.

Before migrating, please see [Breaking or Potentially Breaking Changes](#changes-compared-to-v1) to learn about a few breaking change in the interface.

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


## Breaking or Potentially Breaking Changes

### Global Arguments Come First

Global flags in `rabbitmqadmin` v2 must precede the command category (e.g. `list`) and the command itself,
namely various HTTP API endpoint options and `--vhost`:

```shell
rabbitmqadmin --vhost "events" declare queue --name "target.quorum.queue.name" --type "quorum" --durable true
```

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
