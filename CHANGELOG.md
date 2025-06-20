# rabbitmqadmin-ng Change Log

## v2.3.0 (in development)

No changes yet.


## v2.2.1 (Jun 20, 2025)

### Bug Fixes

 * Several `rabbitmqadmin.conf` settings were not merged correctly with
   the command line arguments.

   GitHub issue: [#58](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/58)


## v2.2.0 (Jun 12, 2025)

### Enhancements

 * `connections` is a new command group for operations on connections
 * `channels` is a new command group for operations on channels
 * `operator_policies` is a new command group for working with operator policies.
   It matches the `policies` group but acts on [operator policies](https://www.rabbitmq.com/docs/policies#operator-policies)
 * `policies set` and `policies update` are two new aliases for `policies declare`. The former follows the naming
   used by `rabbitmqctl` and the latter reflects the fact that the command can be used to update an existing policy,
   in particular, to override its definition
 * `policies patch` is a new command that updates a policy definition by merging the provided definition with the existing one
 * `policies delete_definition_keys` is a new command that removes keys from a policy definition
 * `policies delete_definition_keys_from_all_in` is a new command that removes definition keys from all policies in a virtual host
 * `policies update_definition` is a new command that updates a policy definition key; for multi-key updates, see `policies patch
 * `policies update_definitions_of_all_in` is a new command that updates a definition key for all policies in a virtual host
 * `policies declare_override` is a new command that declares a policy that overrides another policy
 * `policies declare_blanket` is a new command that declares a low priority policy that matches all objects not matched
   by any other policies
 * `parameters list_all` is a new command that lists all runtime parameters across all virtual hosts
 * `parameters list_in` is a new command that lists runtime parameters of a given component (type)
   in a specific virtual host


## v2.1.0 (May 8, 2025)

### Enhancements

 * `bindings` is a new command group for operations on bindings
 * `exchanges` is a new command group for operations on exchanges
 * `global_parameters` is a new command group for operations on [global runtime parameters](https://www.rabbitmq.com/docs/parameters)
 * `nodes` is a new command group for operations on nodes
 * `parameters` is a new command group for operations on [runtime parameters](https://www.rabbitmq.com/docs/parameters)
 * `queues` is a new command group for operations on queues
 * `streams` is a new command group for operations on streams
 * `users` is a new command group for operations on users
 * `vhosts` is a new command group for operations on virtual hosts
 * Command groups are now ordered alphabetically

### Bug Fixes

 * Both `-h` and `--help` now display relevant doc guide URLs.
   Previously it was only the case for `--help`

### Other Changes

 * `vhosts declare` no longer has a default value for `--default-queue-type`.
   Instead, the default will be controlled exclusively by RabbitMQ


## v2.0.0 (Mar 31, 2025)

### Enhancements

#### Subcommand and Long Option Inference

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

#### Intentionally Restricted Environment Variable Support

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
| `RABBITMQADMIN_INFER_SUBCOMMANDS`    | Boolean                                           | Pre-flight (before command execution) | Enables inference (completion of partial names) of subcommands. Does not apply to the non-interactive mode. |
| `RABBITMQADMIN_INFER_LONG_OPTIONS`   | Boolean                                           | Pre-flight (before command execution) | Enables inference (completion of partial names) of `--long-options`. Does not apply to the non-interactive mode. |
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

## v0.29.0 (Mar 23, 2025)

### Breaking Changes

 * `definitions export`'s special `--file` value of `-` for "standard input" is deprecated. Use `--stdout` instead:

   ```shell
   rabbitmqadmin definitions export --stdout > definitions.json
   ```

      ```shell
   # exports 3.x-era definitions that might contain classic queue mirroring keys, transforms
   # them to not use any CMQ policies, injects an explicit queue type into the matched queues,
   # and drops all the policies that had nothing beyond the CMQ keys,
   # then passes the result to the standard input of
   # 'rabbitmqadmin definitions import --stdin'
   rabbitmqadmin --node "source.node" definitions export --transformations strip_cmq_keys_from_policies,drop_empty_policies --stdout | rabbitmqadmin --node "destination.node" definitions import --stdin
   ```

### Enhancements

 * `definitions import` now supports reading definitions from the standard input instead of a file.
   For that, pass `--stdin` instead of `--file "/path/to/definitions.json"`.

   ```shell
   rabbitmqadmin definitions import --stdin < definitions.json
   ```

   ```shell
   cat definitions.json | rabbitmqadmin definitions import --stdin
   ```

   ```shell
   # exports 3.x-era definitions that might contain classic queue mirroring keys, transforms
   # them to not use any CMQ policies, injects an explicit queue type into the matched queues,
   # and drops all the policies that had nothing beyond the CMQ keys,
   # then passes the result to the standard input of
   # 'rabbitmqadmin definitions import --stdin'
   rabbitmqadmin --node "source.node" definitions export --transformations strip_cmq_keys_from_policies,drop_empty_policies --stdout | rabbitmqadmin --node "destination.node" definitions import --stdin
   ```


## v0.28.0 (Mar 23, 2025)

### Enhancements

 * New command group: `federation`, see

   ```shell
   rabbitmqadmin federation help
   ```

* New command: `federation declare_upstream_for_queues` for declaring upstreams that will exclusively be used for queue
  federation. This command does not support any options related to exchange federation.

  ```shell
  rabbitmqadmin federation --vhost "local.vhost" declare_upstream_for_queues \
             --name "dc.vancouver" \
             --uri "amqp://192.168.0.25/demote.vhost" \
             --ack-mode "on-confirm"
  ```

* New command: `federation declare_upstream_for_exchanges` for declaring upstreams that will exclusively be used exchange
  federation. This command does not support any options related to queue federation.

  ```shell
  rabbitmqadmin federation --vhost "local.vhost" declare_upstream_for_exchanges \
             --name "dc.vancouver" \
             --uri "amqp://192.168.0.25/demote.vhost" \
             --ack-mode "on-confirm"
  ```

 * New command: `federation declare_upstream` for declaring upstreams that can be used for either queue or exchange
   federation. This command supports the whole spectrum of federation upstream options, that is, both the settings
   of queue and exchange federation.

   ```shell
   rabbitmqadmin  federation --vhost "local.vhost" declare_upstream \
              --name "dc.canada.bc.vancouver" \
              --uri "amqp://192.168.0.25/demote.vhost" \
              --ack-mode "on-confirm"
   ```

 * New command: `federation list_all_upstreams` for listing all upstreams (that is, upstreams across all the virtual hosts in the cluster).

   ```shell
   rabbitmqadmin federation list_all_upstreams
   ```

 * New command: `federation list_all_links` for listing all links (that is, links across all the virtual hosts in the cluster).

   ```shell
   rabbitmqadmin federation list_all_links
   ```

 * New command: `federation delete_upstream`. As the name suggests, it deletes an upstream.

   ```shell
   rabbitmqadmin federation delete_upstream --name "dc.canada.bc.vancouver"
   ```

 * New definitions export `--transformations` value, `obfuscate_usernames`, changes usernames to dummy values
   (e.g. so that definitions could be shared safely with external teams)
 * New definitions export `--transformations` value, `exclude_users`, removes users from the result
   (also for safe sharing)
 * New definitions export `--transformations` value, `exclude_permissions`, removes all permissions
   (also for safe sharing)
 * New definitions export `--transformations` value, `exclude_runtime_parameters`, removes all runtime parameters
 * New definitions export `--transformations` value, `exclude_policies`, removes all policies
 * New definitions export `--transformations` value, `no_op`, applies no transformation


## v0.27.0 (Mar 10, 2025)

### Enhancements

 * `definitions export` now supports a new option, `--transformations`, a comma-separated list of
   supported operations to apply to the definitions.

   ```shell
   rabbitmqadmin definitions export --transformations strip_cmq_keys_from_policies,drop_empty_policies
   ```

   The command above applies two transformations named `strip_cmq_keys_from_policies` and `drop_empty_policies`
   that will strip all classic queue mirroring-related policy keys that RabbitMQ 3.13 nodes supported,
   then removes the policies that did not have any keys left (ended up having an empty definition).

 * When `--non-interactive` mode is used, newlines in table cells are now replaced with comma-separated lists

### Bug Fixes

 * 'declare queue's `--type` option values that the tool does not recognize are now passed as is to
   the HTTP API


## v0.26.0 (Mar 3, 2025)

### Enhancements

 * `policies` is a new command group for policy operations:

   ```shell
   rabbitmqadmin help policies

   # an equivalent of 'declare policy'
   rabbitmqadmin policies declare --name "policy-name" --pattern '^matching\..+' --apply-to "quorum_queues" \
                                  --priority 10 \
                                  --definition '{"max-length": 10000}'

   # an equivalent of 'list policies'
   rabbitmqadmin policies list

   # an equivalent of 'delete policy'
   rabbitmqadmin policies delete --name "policy-name"
   ```
 * `policies list_in` is a new command that lists policies in a specific virtual host:

   ```shell
   rabbitmqadmin --vhost "a.vhost" policies list_in
   ```

   ```shell
   rabbitmqadmin --vhost "streams.vhost" policies list_in --apply-to "streams"
   ```

 * `policies list_matching_object` is a new command that lists all policies that
   would match an object (a queue, a stream, an exchange)
   with a given name:

   ```shell
   rabbitmqadmin --vhost "a.vhost" policies list_matching_object --name 'audit.events' --type queues
   ```

### Bug Fixes

 * `declare policy`'s `--apply-to` argument value was ignored



## v0.25.0 (Mar 2, 2025)

### Enhancements

 * Binary packages for 8x86-64 Linux are now produced on an older `glibc` version, `2.35`,
   for compatibility with Debian Bookworm and Ubuntu 22.04

 * `shovels declare_amqp10` is a new command that declares a dynamic Shovel that will use
  AMQP 1.0 for both source and destination:

  ```shell
  rabbitmqadmin --vhost "shovels" shovels declare_amqp10 \
                --name "shovel-2" \
                --source-uri amqp://localhost:5672 \
                --destination-uri amqp://localhost:5672 \
                --source-address "/queue/src.q" --destination-address "/queue/dest.q"
  ```

 * `shovels declare_amqp091` is a new command that declares a dynamic Shovel that will use
   AMQP 0-9-1 for both source and destination:

   ```shell
   rabbitmqadmin --vhost "shovels" shovels declare_amqp091 \
                 --name "shovel-2" \
                 --source-uri amqp://localhost:5672 \
                 --destination-uri amqp://localhost:5672 \
                 --source-queue "src.q" --destination-exchange "amq.fanout"
   ```

 * `shovels delete` is a new command that deletes a dynamic shovel:

   ```shell
   rabbitmqadmin --vhost "shovels" shovels delete --name "shovel-2"
   ```


## v0.24.0 (Feb 8, 2025)

### Enhancements

 * `definitions export_from_vhost` is a new command that exports definitions from a single virtual host
   (as opposed to definitions for the entire cluster)

 * `definitions import_into_vhost` is a new command that imports virtual host-specific definitions
   (as opposed to definitions for the entire cluster)


## v0.23.0 (Feb 2, 2025)

### Enhancements

 * `list user_connections` is a new command that lists connections of a specific user:

   ```
   rabbitmqadmin --vhost="/" list user_connections --username "monitoring.1"

   rabbitmqadmin --vhost="production" list user_connections --username "web.45cf7dc28"
   ```

* `close user_connections` is a new command that closes connections of a specific user:

  ```
  rabbitmqadmin --vhost="/" close user_connections --username "monitoring.2"

  rabbitmqadmin --vhost="production" close user_connections --username "web.94ee67772"
  ```

 * New general option `--table-style`, can be used to change output table styling.

   By default, the following style is used:

   ```shell
   rabbitmqadmin --table-style=modern show overview
   ```

   An equivalent of `--non-interactive` in terms of styling is

   ```shell
   rabbitmqadmin --table-style=borderless show overview
   ```

   More available styles:

   ```shell
   rabbitmqadmin --table-style=ascii show overview
   ```

   ```shell
   rabbitmqadmin --table-style=psql show overview
   ```

   ```shell
   rabbitmqadmin --table-style=markdown show overview
   ```

   ```shell
   rabbitmqadmin --table-style=dots show overview
   ```


## v0.22.0 (Feb 1, 2025)

### Naming

* `tanzu sds enable` was renamed to `tanzu sds enable_on_node`.

  This breaking change only applies to a command specific to
  Tanzu RabbitMQ 4.1, a series currently in development.

* `tanzu sds disable` was renamed to `tanzu sds disable_on_node`.

  This breaking change only applies to a command specific to
  Tanzu RabbitMQ 4.1, a series currently in development.

### Enhancements

* `tanzu sds enable_cluster_wide` is a new command that disables SDS on all cluster nodes.

  This command is specific to Tanzu RabbitMQ 4.1, a series currently in development.

* `tanzu sds disable_cluster_wide` is a new command that disables SDS on all cluster nodes.

  This command is specific to Tanzu RabbitMQ 4.1, a series currently in development.



## v0.21.0 (Feb 1, 2025)

### Bug Fixes

 * `list connections` now correctly handles RabbitMQ Stream Protocol
   connections that do not have the `channel_max` metric set

### Enhancements

 * `declare stream` is a new command that accepts stream-specific
   arguments:

   ```shell
   rabbitmqadmin --vhost "vh1" declare stream --name "streams.1" --expiration "8h" \
                                              --arguments '{"x-initial-cluster-size": 3}'
   ```

  * `delete stream` is an alias for `delete queue` that makes more sense for
    environments where [streams](https://www.rabbitmq.com/docs/streams)
    are used more often than queues:

    ```shell
    rabbitmqadmin --vhost "vh1" delete stream --name "streams.1"
    ```


## v0.20.0 (Jan 28, 2025)

### Enhancements

 * Initial support for Tanzu RabbitMQ Schema Definition Sync (SDS).

   ```
   rabbitmqadmin help tanzu sds

   rabbitmqadmin tanzu sds status
   ```

 * Initial support for Tanzu RabbitMQ Warm Standby Replication (WSR).

   ```
   rabbitmqadmin help tanzu wsr

   rabbitmqadmin tanzu wsr status
   ```


## v0.19.0 (Jan 5, 2025)

### Enhancements

* Two new commands for reasoning about target [node's memory footprint](https://www.rabbitmq.com/docs/memory-use):

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

  Note that there are [two different supported strategies](https://www.rabbitmq.com/docs/memory-use#strategies) for computing memory footprint of a node.  RabbitMQ uses both and takes the greater value for 100% when computing the relative
  share in percent for each category. Other factors that can affect the precision of percentage values reported  are [runtime allocator](https://www.rabbitmq.com/docs/memory-use#preallocated-memory) behavior nuances and the [kernel page cache](https://www.rabbitmq.com/docs/memory-use#page-cache).



## v0.18.0 (Jan 1, 2025)

### Enhancements

 * Client identity support: `--tls-cert-file` and `--tls-key-file` are the (re-introduced)
   options that allow the user to pass in a public certificate (key) and private key pair
   for x.509 [peer verification](https://www.rabbitmq.com/docs/ssl#peer-verification):

   ```shell
   rabbitmqadmin --use-tls --host 'target.domain' --port 15671 \
                 --tls-ca-cert-file '/path/to/ca_certificate.pem' \
                 --tls-cert-file '/path/to/client_certificate.pem' \
                 --tls-key-file '/path/to/client_key.pem' \
                 list connections
   ```

   GitHub issue: [#26](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/26)

 * Initial cross-platform support for loading of [trusted CA certificates](https://www.rabbitmq.com/docs/ssl#peer-verification-trusted-certificates)
   from system locations.

   This behavior is enabled automatically. The certificates in a PEM file passed in
   via `--tls-ca-cert-file` are merged with the list of CA certificates discovered in
   the platform-specific stores.

   GitHub issue: [#42](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/42)

 * `rabbitmqadmin show memory_breakdown` is a new command that outputs [a breakdown of target node's memory footprint](https://www.rabbitmq.com/docs/memory-use#breakdown)


## v0.17.0 (Dec 31, 2024)

### Enhancements

 * New [health checks](https://www.rabbitmq.com/docs/monitoring#health-checks):

   ```shell
   # To see help: 'rabbitmqadmin health_check help port_listener'
   rabbitmqadmin health_check port_listener --port [port]
   ```

   ```shell
   # To see help: 'rabbitmqadmin health_check help protocol_listener'
   rabbitmqadmin health_check protoocl_listener --protocol [protocol]
   ```


## v0.16.0 (Dec 29, 2024)

### Enhancements

 * `rabbitmqadmin feature_flags list` (also available as `rabbitmqadmin list feature_flags`) is a new command
    that lists [feature flags](https://www.rabbitmq.com/docs/feature-flags) and their cluster state.

    GitHub issue: [#38](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/38)

* `rabbitmqadmin feature_flags enable --name {feature flag}` and `rabbitmqadmin feature_flags enable_all` are new commands
  that enable feature flags.

  Just like its `rabbitmqctl` counterpart, `rabbitmqadmin feature_flags enable_all` will only enable
  the stable [feature flags](https://www.rabbitmq.com/docs/feature-flags) and will skip the experimental ones.

  GitHub issues: [#41](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/41)

 * `rabbitmqadmin deprecated_features list` (also available as `rabbitmqadmin list deprecated_features`) is a new
    function that lists all [deprecated features](https://www.rabbitmq.com/docs/deprecated-features).

    GitHub issue: [#39](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/39)

* `rabbitmqadmin deprecated_features list_used` (also available as `rabbitmqadmin list deprecated_features_in_use`) is a new
  function that lists the [deprecated features](https://www.rabbitmq.com/docs/deprecated-features) that are found to be
  used in the cluster.

  GitHub issue: [#40](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/40)


## v0.15.0 (Dec 26, 2024)

### Enhancements

 * Improved error reporting.

   A failed HTTP API request now prints the request URL (this does NOT include the credentials),
   and the response body, making it easier to identify the problem without consulting [RabbitMQ node logs](https://www.rabbitmq.com/docs/management#http-logging).

 * CLI interface `help` message polishing.

   More commands now provide links to the relevant documentation guides,
   use (a reasonable amount of) coloring and recommend against features such as that are
   [polling message consumption]((https://www.rabbitmq.com/docs/consumers#polling)) that were never designed or intended to be used in production

 * README documentation improvements



## v0.14.0 (Dec 22, 2024)

### Breaking Changes

 * Multi-word command line flags now use the more common `--snake-case[=]{value}` format
   instead of `rabbitmqadmin` v1's `lower_case={value}`.

### Enhancements

 * New command category: `health_check` which provides access to the [health check endpoints](https://rabbitmq.com/docs/monitoring#health-checks).

   Currently, only the three (arguably) most important health checks are implemented: `local_alarms`, `cluster_wide_alarms`, and `node_is_quorum_critical`.

   GitHub issues: [#33](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/33), [#34](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/34).

 * `health_check help` now includes a link to the respective RabbitMQ documentation guide.

### Bug Fixes

* Configuration paths with a tilda (`~`), including the  default configuration file at `~/.rabbitmqadmin.conf`,
  were not loaded correctly.


## v0.13.0 (Dec 21, 2024)

### Enhancements

 * Several key `delete` commands, namely `delete vhost`, `delete user`, `delete queue` and `delete exchange` now support a new flag, `--idempotently`.
   When this flag is used, 404 Not Found responses from the HTTP

   GitHub issue: [#32](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/32)

 * `--non-interactive` is a new global flag. When used, this flag will instruct `rabbitmqadmin` to not produce table border formatting,
    and generally try to use output that'd be easier to consume from scripts

 * Initial work on improving error reporting

## Bug Fixes

 * `declare exchange` produced an incorrect API request payload
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
   with the [default queue type of the target virtual host](https://rabbitmq.com/docs/vhosts#default-queue-type).

   GitHub issue: [#29](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/29)

### Releases

Release artifacts are no longer distributed as single file archives. Instead,
the release now includes "naked" binaries that can be downloaded and executed
without un-archiving.

GitHub issue: [#31](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/31)
