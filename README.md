# rabbitmqadmin v2: a Modern Command Line Client for the [RabbitMQ HTTP API](https://www.rabbitmq.com/docs/management#http-api)

`rabbitmqadmin` v2 is a major revision of `rabbitmqadmin`, one of the [RabbitMQ CLI tools](https://www.rabbitmq.com/docs/cli)
that target the [HTTP API](https://www.rabbitmq.com/docs/management#http-api).

If you are migrating from the original `rabbitqadmin`, please see [Breaking or Potentially Breaking Changes](#breaking-or-potentially-breaking-changes)
to learn about the breaking changes in the command line interface.

The general "shape and feel" of the interface is still very similar to `rabbitmqadmin` v1.


## Supported RabbitMQ Series

`rabbitmqadmin` v2 targets

 * Open source RabbitMQ `4.x`
 * Open source RabbitMQ `3.13.x` (specifically for the command groups and commands related to upgrades)
 * Tanzu RabbitMQ `4.x`
 * Tanzu RabbitMQ `3.13.x`


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

For usage documentation, see the [dedicated RabbitMQ doc guide](https://www.rabbitmq.com/docs/management-cli) and/or [Usage](#usage) below.


## Getting Help

Please use GitHub Discussions in this repository and [RabbitMQ community Discord server](https://rabbitmq.com/discord/).


## Project Maturity

This version of `rabbitmqadmin` should be considered reasonably mature to be used.

Before migrating, please see [Breaking or Potentially Breaking Changes](#breaking-or-potentially-breaking-changes) to learn about a few breaking change in the interface.


## Usage

### Exploring Available Command Groups and Sub-commands

To explore what command groups are available, use

```shell
rabbitmqadmin help
```

which will output a list of command groups:

```
Usage: rabbitmqadmin [OPTIONS] <COMMAND>

Commands:
  show                 Overview, memory footprint breakdown, and more
  list                 Lists objects
  declare              Creates or declares objects
  delete               Deletes objects
  purge                Purges queues
  policies             Operations on policies
  health_check         Runs health checks
  close                Closes connections
  rebalance            Rebalancing of leader replicas
  definitions          Operations on definitions (everything except for messages: virtual hosts, queues, streams, exchanges, bindings, users, etc)
  export               See 'definitions export'
  import               See 'definitions import'
  feature_flags        Operations on feature flags
  deprecated_features  Operations on deprecated features
  publish              Publishes (inefficiently) message(s) to a queue or a stream. Only suitable for development and test environments.
  get                  Fetches message(s) from a queue or stream via polling. Only suitable for development and test environments.
  shovels              Operations on shovels
  federation           Operations on federation upstreams and links
  tanzu                Tanzu RabbitMQ-specific commands
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
# => declares a queue or a stream
# =>
# => Usage: rabbitmqadmin declare queue [OPTIONS] --name <name>
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
┌───────────────────────────────────────────────────────────────────┬─────────────────────────────────────────────────────────────────────────────────────────────────┐
│ Overview                                                                                                                                                            │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ key                                                               │ value                                                                                           │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Product name                                                      │ RabbitMQ                                                                                        │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Product version                                                   │ 4.0.5                                                                                           │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ RabbitMQ version                                                  │ 4.0.5                                                                                           │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Erlang version                                                    │ 27.2.1                                                                                          │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Erlang details                                                    │ Erlang/OTP 27 [erts-15.2.1] [source] [64-bit] [smp:10:10] [ds:10:10:10] [async-threads:1] [jit] │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Connections (total)                                               │ 4                                                                                               │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ AMQP 0-9-1 channels (total)                                       │ 4                                                                                               │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Queues and streams (total)                                        │ 4                                                                                               │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Consumers (total)                                                 │ 4                                                                                               │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Messages (total)                                                  │ 222                                                                                             │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Messages ready for delivery (total)                               │ 2                                                                                               │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Messages delivered but unacknowledged by consumers (total)        │ 220                                                                                             │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Publishing (ingress) rate (global)                                │                                                                                                 │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Publishing confirm rate (global)                                  │                                                                                                 │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Consumer delivery (egress) rate (global)                          │                                                                                                 │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Consumer delivery in automatic acknowledgement mode rate (global) │                                                                                                 │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Consumer acknowledgement rate (global)                            │                                                                                                 │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Unroutable messages: returned-to-publisher rate (global)          │                                                                                                 │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Unroutable messages: dropped rate (global)                        │                                                                                                 │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Cluster tags                                                      │ "az": "us-east-3"                                                                               │
│                                                                   │ "environment": "production"                                                                     │
│                                                                   │ "region": "us-east"                                                                             │
│                                                                   │                                                                                                 │
├───────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Node tags                                                         │ "environment": "production"                                                                     │
│                                                                   │ "instance": "xlarge.m3"                                                                         │
│                                                                   │                                                                                                 │
└───────────────────────────────────────────────────────────────────┴─────────────────────────────────────────────────────────────────────────────────────────────────┘
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
 Product version   4.1.0
 RabbitMQ version  4.1.0
 Erlang version    26.2.5.10
 Erlang details    Erlang/OTP 26 [erts-14.2.5.9] [source] [64-bit] [smp:10:10] [ds:10:10:10] [async-threads:1] [jit]
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
rabbitmqadmin --vhost "events" declare queue --name "target.classic.queue.name" --type "classic" --durable true --auto-delete false
```

### Purge a queue

```
rabbitmqadmin --vhost "events" purge queue --name "target.queue.name"
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

### Inspecting Node Memory Breakdown

There are two commands for reasoning about target [node's memory footprint](https://rabbitmq.com/docs/memory-use):

```shell
# displays a breakdown in bytes
rabbitmqadmin show memory_breakdown_in_bytes --node 'rabbit@hostname'
```

```shell
# displays a breakdown in percent
rabbitmqadmin show memory_breakdown_in_percent --node 'rabbit@hostname'
```

Example output of `show memory_breakdown_in_percent`:

```
┌────────────────────────────────────────┬────────────┐
│ key                                    │ percentage │
├────────────────────────────────────────┼────────────┤
│ total                                  │ 100%       │
├────────────────────────────────────────┼────────────┤
│ Binary heap                            │ 45.10%     │
├────────────────────────────────────────┼────────────┤
│ Allocated but unused                   │ 23.45%     │
├────────────────────────────────────────┼────────────┤
│ Quorum queue ETS tables                │ 23.05%     │
├────────────────────────────────────────┼────────────┤
│ Other processes                        │ 5.32%      │
├────────────────────────────────────────┼────────────┤
│ Other (used by the runtime)            │ 4.98%      │
├────────────────────────────────────────┼────────────┤
│ Code                                   │ 4.54%      │
├────────────────────────────────────────┼────────────┤
│ Client connections: others processes   │ 3.64%      │
├────────────────────────────────────────┼────────────┤
│ Management stats database              │ 3.48%      │
├────────────────────────────────────────┼────────────┤
│ Client connections: reader processes   │ 3.22%      │
├────────────────────────────────────────┼────────────┤
│ Plugins and their data                 │ 3.12%      │
├────────────────────────────────────────┼────────────┤
│ Other (ETS tables)                     │ 1.55%      │
├────────────────────────────────────────┼────────────┤
│ Metrics data                           │ 0.66%      │
├────────────────────────────────────────┼────────────┤
│ AMQP 0-9-1 channels                    │ 0.40%      │
├────────────────────────────────────────┼────────────┤
│ Message store indices                  │ 0.27%      │
├────────────────────────────────────────┼────────────┤
│ Atom table                             │ 0.24%      │
├────────────────────────────────────────┼────────────┤
│ Client connections: writer processes   │ 0.19%      │
├────────────────────────────────────────┼────────────┤
│ Quorum queue replica processes         │ 0.10%      │
├────────────────────────────────────────┼────────────┤
│ Stream replica processes               │ 0.07%      │
├────────────────────────────────────────┼────────────┤
│ Mnesia                                 │ 0.02%      │
├────────────────────────────────────────┼────────────┤
│ Metadata store                         │ 0.02%      │
├────────────────────────────────────────┼────────────┤
│ Stream coordinator processes           │ 0.02%      │
├────────────────────────────────────────┼────────────┤
│ Classic queue processes                │ 0.00%      │
├────────────────────────────────────────┼────────────┤
│ Metadata store ETS tables              │ 0.00%      │
├────────────────────────────────────────┼────────────┤
│ Stream replica reader processes        │ 0.00%      │
├────────────────────────────────────────┼────────────┤
│ Reserved by the kernel but unallocated │ 0.00%      │
└────────────────────────────────────────┴────────────┘
```

Note that there are [two different supported strategies](https://rabbitmq.com/docs/memory-use#strategies)
for computing memory footprint of a node. `rabbitmqadmin` will use the greater value
for 100% when computing the relative share in percent for each category.

Other factors that can affect the precision of percentage values reported
are [runtime allocator](https://rabbitmq.com/docs/memory-use#preallocated-memory)
behavior nuances and the [kernel page cache](https://rabbitmq.com/docs/memory-use#page-cache).

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

### Export Definitions

To export [definitions](https://www.rabbitmq.com/docs/definitions) to standard output, use `definitions export --stdout`:

```shell
rabbitmqadmin definitions export --stdout
```

To export definitions to a file, use `definitions export --file /path/to/definitions.file.json`:

```shell
rabbitmqadmin definitions export --file /path/to/definitions.file.json
```

### Export and Transform Definitions

`definitions export` can transform the exported JSON definitions file it gets from the
target node. This is done by applying one or more transformations to the exported
JSON file.

This can be useful to remove classic queue mirroring-related keys (such as `ha-mode`) from a definitions
set originating from a 3.13.x node, or to obfuscate usernames and passwords, or exclude certain definitions file
sections entirely.

To specify what transformations should be applied, use the `--transformations` options,
which takes a comma-separated list of  supported operation names.

The following table explains what transformations are available and what they do:

| Transformation name            | Description                                                  |
|--------------------------------|--------------------------------------------------------------|
| `strip_cmq_keys_from_policies` | Deletes all classic queue mirroring-related keys (such as `ha-mode`) from all exported policies.<br><br>Must be followed by `drop_empty_policies` to strip off the policies whose definition has become empty (and thus invalid at import time) after the removal of all classic queue mirroring-related keys |
| `drop_empty_policies`          | Should be used after `strip_cmq_keys_from_policies` to strip off the policies whose definition has become empty (and thus invalid at import time) after the removal of all classic queue mirroring-related keys |
| `obfuscate_usernames`          | Replaces usernames and passwords with dummy values.<br><br>For usernames the values used are: `obfuscated-username-1`, `obfuscated-username-2`, and so on.<br><br>For passwords the values generated are: `password-1`, `password-2`, and so forth.<br><br>This transformations updates both the users and the permissions sections, consistently |
| `exclude_users`                | Removes all users from the result. Commonly used together with `exclude_permissions` |
| `exclude_permissions`          | Removes all permissions from the result. Commonly used together with `exclude_users` |
| `exclude_runtime_parameters`   | Removes all runtime parameters (including federation upstreams, shovels, WSR and SDS settings in Tanzu RabbitMQ) from the result |
| `exclude_policies`             | Removes all policies from the result                         |
| `no_op`                        | Does nothing. Can be used as the default in dynamically computed transformation lists, e.g. in scripts |

#### Examples

The following command applies two transformations named `strip_cmq_keys_from_policies` and `drop_empty_policies`
that will strip all classic queue mirroring-related policy keys that RabbitMQ 3.13 nodes supported,
then removes the policies that did not have any keys left (ended up having an empty definition):

```shell
# strips classic mirrored queue-related policy keys from the exported definitions, then prints them
# to the standard output stream
rabbitmqadmin definitions export --stdout --transformations strip_cmq_keys_from_policies,drop_empty_policies
```

The following example exports definitions without users and permissions:

```shell
# removes users and user permissions from the exported definitions, then prints them
# to the standard output stream
rabbitmqadmin definitions export --stdout --transformations exclude_users,exclude_permissions
```

To export definitions with usernames replaced by dummy values (usernames: `obfuscated-username-1`, `obfuscated-username-2`, and so on;
passwords: `password-1`, `password-2`, and so forth), use the `obfuscate_usernames` transformation:

```shell
rabbitmqadmin definitions export --file /path/to/definitions.file.json --transformations obfuscate_usernames
```

### Import Definition

To import definitions from the standard input, use `definitions import --stdin`:

```shell
cat /path/to/definitions.file.json | rabbitmqadmin definitions import --stdin
```

To import definitions from a file, use `definitions import --file /path/to/definitions.file.json`:

```shell
rabbitmqadmin definitions import --file /path/to/definitions.file.json
```

### Declare an AMQP 0-9-1 Shovel

To declare a [dynamic shovel](https://www.rabbitmq.com/docs/shovel-dynamic) that uses AMQP 0-9-1 for both source and desitnation, use
`shovel declare_amqp091`:

```shell
rabbitmqadmin shovel declare_amqp091 --name my-amqp091-shovel \
    --source-uri amqp://username:s3KrE7@source.hostname:5672 \
    --destination-uri amqp://username:s3KrE7@source.hostname:5672 \
    --ack-mode "on-confirm" \
    --source-queue "src.queue" \
    --destination-queue "dest.queue" \
    --predeclared-source false \
    --predeclared-destination false
```

### Declare an AMQP 1.0 Shovel

To declare a [dynamic shovel](https://www.rabbitmq.com/docs/shovel-dynamic) that uses AMQP 1.0 for both source and desitnation, use
`shovel declare_amqp10`.

Note that

1. With AMQP 1.0 shovels, credentials in the URI are mandatory (there are no defaults)
2. With AMQP 1.0 shovels, the topology must be pre-declared (an equivalent of `--predeclared-source true` and `--predeclared-destination true` for AMQP 0-9-1 shovels)
2. AMQP 1.0 shovels should use [AMQP 1.0 addresses v2](https://www.rabbitmq.com/docs/amqp#addresses)

```shell
rabbitmqadmin shovel declare_amqp10 --name my-amqp1.0-shovel \
    --source-uri "amqp://username:s3KrE7@source.hostname:5672?hostname=vhost:src-vhost" \
    --destination-uri "amqp://username:s3KrE7@source.hostname:5672?hostname=vhost:dest-vhost" \
    --ack-mode "on-confirm" \
    --source-address "/queues/src.queue" \
    --destination-address "/queues/dest.queue"
```

### List Shovels

To list shovels across all virtual hosts, use `shovel list_all`:

```shell
rabbitmqadmin shovel list_all
```

### Delete a Shovel

To delete a shovel, use `shovel delete --name`:

```shell
rabbitmqadmin shovel delete --name my-amqp091-shovel
```

### List Federation Upstreams

To list [federation upstreams](https://www.rabbitmq.com/docs/federation) across all virtual hosts, use `federation list_all_upstreams`:

```shell
rabbitmqadmin federation list_all_upstreams
```

### Create a Federation Upstream for Exchange Federation

To create a [federation upstream](https://www.rabbitmq.com/docs/federated-exchanges), use `federation declare_upstream_for_exchanges`

```shell
rabbitmqadmin --vhost "local-vhost" federation declare_upstream_for_exchanges --name "pollux" \
                --uri "amqp://pollux.eng.megacorp.local:5672/remote-vhost" \
                --ack-mode 'on-publish' \
                --prefetch-count 2000 \
                --exchange-name "overridden.name" \
                --queue-type quorum \
                --bind-using-nowait true
```

### Create a Federation Upstream for Queue Federation

To create a [federation upstream](https://www.rabbitmq.com/docs/federated-queues), use `declare_upstream_for_queues`.
This command provides a reduced set of options, only those that are relevant
specifically to queue federation.

```shell
rabbitmqadmin --vhost "local-vhost" federation declare_upstream_for_queues --name "clusters.sirius" \
                --uri "amqp://sirius.eng.megacorp.local:5672/remote-vhost" \
                --ack-mode 'on-publish' \
                --prefetch-count 2000 \
                --queue-name "overridden.name" \
                --consumer-tag "overriden.ctag"
```

### Create a Universal Federation Upstream

To create a [federation upstream](https://www.rabbitmq.com/docs/federation) that will be (or can be)
used for federating both queues and exchanges, use `declare_upstream`. It combines
[all the federation options](https://www.rabbitmq.com/docs/federation-reference), that is,
the options of both `declare_upstream_for_queues` and `declare_upstream_for_exchanges`.

```shell
rabbitmqadmin --vhost "local-vhost" federation declare_upstream --name "pollux" \
                --uri "amqp://pollux.eng.megacorp.local:5672/remove-vhost" \
                --ack-mode 'on-publish' \
                --prefetch-count 2000 \
                --queue-name "overridden.name" \
                --consumer-tag "overriden.ctag" \
                --exchange-name "overridden.name" \
                --queue-type quorum \
                --bind-using-nowait true
```

### Delete a Federation Upstream

To delete a [federation upstream](https://www.rabbitmq.com/docs/federation), use 'federation delete_upstream',
which takes a virtual host and an upstream name:

```shell
rabbitmqadmin --vhost "local-vhost" federation delete_upstream --name "upstream.to.delete"
```

### List Federation Links

To list all [federation links](https://www.rabbitmq.com/docs/federation) across all virtual hosts, use `federation list_all_links`:

```shell
rabbitmqadmin federation list_all_links
```


## Subcommand and Long Option Inference

This feature is available only in the `main` branch
at the moment.

If the `RABBITMQADMIN_NON_INTERACTIVE_MODE` is not set to `true`, this tool
now can infer subcommand and --long-option names.

This means that a subcommand can be referenced with its unique prefix,
that is,

* 'del queue' will be inferred as 'delete queue'
* 'del q --nam "a.queue"' will be inferred as 'delete queue --name "a.queue"'

To enable each feature, set the following environment variables to
'true':

* `RABBITMQADMIN_INFER_SUBCOMMANDS`
* `RABBITMQADMIN_INFER_LONG_OPTIONS`

This feature is only meant to be used interactively. For non-interactive
use, it can be potentially too dangerous to allow.


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

## Intentionally Restricted Environment Variable Support

Environment variables have a number of serious downsides compared to a `rabbitmqadmin.conf`
and the regular `--long-options` on the command line:

1. Non-existent support for value types and validation ("everything is a string")
2. Subprocess inheritance restrictions that can be very time-consuming to debug
3. Different syntax for setting them between the classic POSIX-era shells (such as `bash`, `zsh`) and modern ones (such as [`nushell`](https://www.nushell.sh/))

For these reasons and others, `rabbitmqadmin` v2 intentionally uses the configuration file and the
CLI options over the environment variables.

`rabbitmqadmin` v2 does, however, supports a number of environment variables for a few
global settings that cannot be configured any other way (besides a CLI option),
or truly represent an environment characteristic, e.g. either the non-interactive mode
should be enabled.

These environment variables are as follows:

| Environment variable                 | Type                                              | When used                             | Description                                                  |
|--------------------------------------|---------------------------------------------------|---------------------------------------|--------------------------------------------------------------|
| `RABBITMQADMIN_CONFIG_FILE_PATH`     | Local filesystem path                             | Pre-flight (before command execution) | Same meaning as the global `--confg-file` argument           |
| `RABBITMQADMIN_NON_INTERACTIVE_MODE` | Boolean                                           | Command execution                     | Enables the non-interactive mode.<br><br>Same meaning as the global `--non-interactive` argument |
| `RABBITMQADMIN_QUIET_MODE`<br>       | Boolean                                           | Command execution                     | Instructs the tool to produce less output.<br><br>Same meaning as the global `--quiet` argument |
| `RABBITMQADMIN_NODE_ALIAS`           | String                                            | Command execution                     | Same meaning as the global `--node` argument                 |
| `RABBITMQADMIN_TARGET_HOST`          | String                                            | Command execution                     | Same meaning as the global `--host` argument                 |
| `RABBITMQADMIN_TARGET_PORT`          | Positive integer                                  | Command execution                     | Same meaning as the global `--port` argument                 |
| `RABBITMQADMIN_API_PATH_PREFIX`      | String                                            | Command execution                     | Same meaning as the global `--path-prefix` argument          |
| `RABBITMQADMIN_TARGET_VHOST`         | String                                            | Command execution                     | Same meaning as the global `--vhost` argument                |
| `RABBITMQADMIN_BASE_URI`             | String                                            | Command execution                     | Same meaning as the global `--base-uri` argument             |
| `RABBITMQADMIN_USE_TLS`              | Boolean                                           | Command execution                     | Same meaning as the global `--tls` argument                  |
| `RABBITMQADMIN_USERNAME`             | String                                            | Command execution                     | Same meaning as the global `--username` argument             |
| `RABBITMQADMIN_PASSWORD`             | String                                            | Command execution                     | Same meaning as the global `--password` argument             |
| `RABBITMQADMIN_TABLE_STYLE`          | Enum, see `--table-style` in `rabbitmqadmin help` | Command execution                     | Same meaning as the global `--table-style` argument          |



## Project Goals Compared to `rabbitmqadmin` v1

This version of `rabbitmqadmin` has a few ideas in mind:

* This is a major version bump. Therefore, reasonable breaking changes are OK. `rabbitmqadmin` hasn't seen a revision in fifteen years
* Some features in `rabbitmqadmin` v1 arguably should never have been built-ins,
  external tools for data processing and [modern shells](https://www.nushell.sh/) can manipulate tabular data
  better than `rabbitmqadmin` ever would
* `rabbitmqadmin` should be standalone binary. There are very few reasons not to build and distribute it that way
* Standalone project, not an obscure feature: `rabbitmqadmin` should be a standalone tool, not a relatively unknown "feature" of
  the RabbitMQ management plugin, and should be developed as such, not tied completely to the development
  environment, practices and release schedule of RabbitMQ itself
* v2 should be a distributed via GitHub releases and not a special `rabbitmq_management` endpoint
* There is a lot of room to improve validation of flags and arguments, since breaking changes are OK for v2
* This tool should strive to be as free as practically possible from CVEs in other projects that show up on security scans.
  CVEs from older Python versions should not plague OCI images that choose to include `rabbitmqadmin` v2


## Breaking or Potentially Breaking Changes

### Some Non-Essential Features Were Dropped

`rabbitmqadmin` v2 does not support

 * Sorting of results. Instead, use `--non-interactive` and parse the spaces-separated
   output. Many modern tools for working with data parse it into a table, sort the data set,
   filter the results, and son. In fact, these features for data processing are ready available [in some shells](https://www.nushell.sh/)
 * Column selection. This feature may be reintroduced
 * JSON output for arbitrary commands (with the exception of `definitions` commands).
   Use the HTTP API directly if you need to work with JSON
 * CSV output for arbitrary commands. This format may be reintroduced

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


## License

This tool, `rabbitmqadmin` (v2 and later versions), is dual-licensed under
the Apache Software License 2.0 and the MIT license.
