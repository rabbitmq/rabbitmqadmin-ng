# rabbitmqadmin-ng Change Log

## v2.25.0 (Feb 5, 2026)

### Enhancements

 * `permissions`, `user_limits`, and their old-style equivalents (`declare permissions`, `delete permissions`,
   `declare user_limit`, `delete user_limit`, `list user_limits`) now use `--username` instead of `--user`:

   ```shell
   rabbitmqadmin permissions declare --username "user1" --configure ".*" --read ".*" --write ".*"
   rabbitmqadmin permissions delete --username "user1"

   rabbitmqadmin user_limits declare --username "user1" --name "max-connections" --value "100"
   rabbitmqadmin user_limits delete --username "user1" --name "max-connections"
   rabbitmqadmin user_limits list --username "user1"

   # old-style (verb-style command group) equivalents
   rabbitmqadmin declare permissions --username "user1" --configure ".*" --read ".*" --write ".*"
   rabbitmqadmin declare user_limit --username "user1" --name "max-connections" --value "100"
   ```

   `--user` is still accepted as a hidden backwards-compatible alias


## v2.24.0 (Feb 2, 2026)

### Enhancements

 * `shell completions` generates shell completion scripts for Nu shell, bash, zsh, fish, elvish:

   ```shell
   # Generate completions for a specific shell
   rabbitmqadmin shell completions --shell nushell
   rabbitmqadmin shell completions --shell zsh
   rabbitmqadmin shell completions --shell bash
   rabbitmqadmin shell completions --shell fish
   rabbitmqadmin shell completions --shell elvish

   # When invoked without --shell, tries to detect the shell using the SHELL environment variable
   # and if that fails, defaults to bash
   rabbitmqadmin shell completions
   ```

### Internal Changes

 * Release infrastructure: adopt [`michaelklishin/rust-release-action`](https://github.com/michaelklishin/rust-release-action)

### Dependency Upgrades

 * `bel7-cli` upgraded to `0.8.0`
 * New dependencies: `clap_complete` at `4.5` and `clap_complete_nushell` at `4.5`


## v2.23.0 (Jan 16, 2026)

### Enhancements

 * RabbitMQ 3.12.x and 3.13.x compatibility for federation commands: `--queue-type` is now optional
 * RabbitMQ 3.12.x compatibility for shovel commands: `--src-predeclared` and `--dest-predeclared` are no longer included
   into RabbitMQ HTTP API requests when set to `false`

### Dependency Upgrades

 * `rabbitmq_http_client` upgraded to `0.76.0`


## v2.22.0 (Jan 15, 2026)

### Enhancements

 * `policies delete_definition_keys_from_all` is an equivalent of `policies delete_definition_keys_from_all_in`
   that deletes definition keys from all policies across all virtual hosts in the cluster:

   ```shell
   rabbitmqadmin policies delete_definition_keys_from_all --definition-keys federation-upstream-set
   ```

   For example, this command can be used to temporarily disable federation across all virtual hosts.

 * `policies update_definitions_of_all` is an equivalent of `policies update_definitions_of_all_in`
   that updates definitions of all policies across all virtual hosts in the cluster:

   ```shell
   rabbitmqadmin policies update_definitions_of_all --definition-key federation-upstream-set --new-value all
   ```

   Similarly to `policies delete_definition_keys_from_all`, this command can be used to [re-]enable federation
   across all virtual hosts.


## v2.21.0 (Jan 6, 2026)

### Breaking Changes

 * When `--tls-ca-cert-file` is provided, only that CA is trusted (previously it was added to the system trust store)

### Enhancements

 * `queues list` now supports `--columns` for selecting specific columns to display:

   ```shell
   rabbitmqadmin queues list --columns name,queue_type,message_count

   rabbitmqadmin queues list --columns name
   ```

   Column names are case-insensitive. Unknown columns are silently ignored.

 * `queues show` is a new command that displays select metrics of a single queue:

   ```shell
   rabbitmqadmin queues show --name "events.incoming"

   rabbitmqadmin queues show --name "orders.pending" --columns name,queue_type,message_count
   ```

 * `streams list` now supports `--columns` for consistency with `queues list`:

   ```shell
   rabbitmqadmin streams list --columns name,queue_type
   ```

 * `streams show` is a stream equivalent of `queues show`:

   ```shell
   rabbitmqadmin streams show --name "events.stream"

   rabbitmqadmin streams show --name "events.stream" --columns name,queue_type
   ```

 * `config_file` is a new command group for operations on `rabbitmqadmin` configuration files:

   ```shell
   rabbitmqadmin config_file show_path

   # Show all configured nodes (passwords masked by default)
   rabbitmqadmin config_file show
   rabbitmqadmin config_file show --reveal-passwords

   # Add a new node (fails if an entry with this name already exists)
   rabbitmqadmin config_file add_node --node experiment-001 --host rabbit.eng.example.com --port 15672 --username admin --password secret --vhost /

   # Update an existing node (or create one if it does not exist).
   # Only the specified fields are updated; unspecified fields are preserved.
   rabbitmqadmin config_file update_node --node experiment-001 --host new-rabbit.eng.example.com --port 15673

   # Enable TLS for a node (other settings like username, password are preserved)
   rabbitmqadmin config_file update_node --node experiment-001 --use-tls --port 15671

   # Disable TLS for a node (other settings are preserved)
   rabbitmqadmin config_file update_node --node experiment-001 --scheme http --port 15672

   # Delete a node (a configuration file entry)
   rabbitmqadmin config_file delete_node --node experiment-001
   ```

### Bug Fixes

 * Fixed a few copy-paste errors in command descriptions in the `stream` group

### Dependency Upgrades

 * `rabbitmq_http_client` upgraded to `0.73.0`
 * `reqwest` upgraded to `0.13.x`


## v2.20.0 (Dec 13, 2025)

### Bug Fixes

 * `--tls-ca-cert-file` was not correctly applied, causing [peer verification](https://www.rabbitmq.com/docs/ssl#peer-verification)
   of connections to TLS-enabled endpoints that don't share the same CA chain as the client to fail with an `UnknownIssuer`


## v2.19.0 (Dec 13, 2025)

### Enhancements

 * Binary releases that use statically linked MUSL for Alpine Linux
 * `rabbitmqadmin` users now can build the tool with native TLS support (via the `native-tls` feature, on by default)
   or Rustls, for example, when targeting platforms that do not provide a dynamically linkable TLS library

### Bug Fixes

 * `definitions export` and `definitions export_from_vhost` now exit with an error code should a file write fail

### Upgrades

 * RabbitMQ HTTP API client was upgraded to [`0.71.0`](https://github.com/michaelklishin/rabbitmq-http-api-rs/releases/tag/v0.71.0)


## v2.18.0 (Dec 11, 2025)

### Enhancements

 * `--page` and `--page-size` options for paginated listing of queues, streams, and connections

### Upgrades

 * RabbitMQ HTTP API client was upgraded to [`0.70.0`](https://github.com/michaelklishin/rabbitmq-http-api-rs/releases/tag/v0.70.0)


## v2.17.0 (Nov 29, 2025)

### Enhancements

 * New command, `auth_attempts stats`, displays authentication attempt statistics per protocol:

   ```
   rabbitmqadmin auth_attempts stats --node rabbit@target.hostname
   ```

### Upgrades

* RabbitMQ HTTP API client was upgraded to [`0.68.0`](https://github.com/michaelklishin/rabbitmq-http-api-rs/releases/tag/v0.68.0)


## v2.16.0 (Oct 20, 2025)

### Enhancements

* `plugins` is a new command group for listing enabled plugins:

  ```shell
  # List plugins across all cluster nodes
  rabbitmqadmin plugins list_all

  # List plugins on a specific node
  rabbitmqadmin plugins list_on_node --node rabbit@hostname
  ```

* Errors now include the `error` or `reason` field from the API response (if they were present there)

* `--timeout` is a new global option limits HTTP API request execution timeout. The value is in seconds and defaults
  to 60s:

  ```shell
  rabbitmqadmin --timeout 15 queues list
  ```

### Upgrades

* RabbitMQ HTTP API client was upgraded to [`0.66.0`](https://github.com/michaelklishin/rabbitmq-http-api-rs/releases/tag/v0.66.0)


## v2.15.0 (Sep 30, 2025)

### Enhancements

* `permissions` is a new command group for operations on user permissions:

  ```shell
  rabbitmqadmin permissions list

  rabbitmqadmin permissions declare --user "user1" --configure ".*" --read ".*" --write ".*"

  rabbitmqadmin permissions delete --user "user1"
  ```

* `user_limits` is a new command group for operations on [per-user limits](https://www.rabbitmq.com/docs/user-limits):

  ```shell
  rabbitmqadmin user_limits list

  rabbitmqadmin user_limits declare --user "user1" --name "max-connections" --value "100"

  rabbitmqadmin user_limits delete --user "user1" --name "max-connections"
  ```

* `vhost_limits` is a new command group for operations on [virtual host limits](https://www.rabbitmq.com/docs/vhosts#limits):

  ```shell
  rabbitmqadmin vhost_limits list

  rabbitmqadmin vhost_limits declare --name "max-connections" --value "1000"

  rabbitmqadmin vhost_limits delete --name "max-connections"
  ```

### Deprecations

* "Verb" command groups (`list [object]`, `declare [object]`, `delete [object]`) are now deprecated in favor of the "noun" group commands (such as `users [operation]` or `permissions [operation]`).


## v2.14.0 (Sep 30, 2025)

### Enhancements

* Several commands now have minimalistic progress indicators: `federation disable_tls_peer_verification_for_all_upstreams`, `federation enable_tls_peer_verification_for_all_upstreams`, `shovels disable_tls_peer_verification_for_all_source_uris`, `shovels disable_tls_peer_verification_for_all_destination_uris`, `shovels enable_tls_peer_verification_for_all_source_uris`, and `shovels enable_tls_peer_verification_for_all_destination_uris`

* `vhosts delete_multiple` is a new command that deletes multiple virtual hosts matching a regular expression pattern:

  ```shell
  # Delete all virtual hosts matching a pattern (requires explicit approval)
  rabbitmqadmin vhosts delete_multiple --name-pattern "test-.*" --approve

  # Dry-run to see what would be deleted without actually deleting
  rabbitmqadmin vhosts delete_multiple --name-pattern "staging-.*" --dry-run

  # Non-interactive mode (no --approve flag needed)
  rabbitmqadmin --non-interactive vhosts delete_multiple --name-pattern "temp-.*"
  ```

  One virtual host — named `/`, that is, the default one — is always skipped to preserve
  at least one functional virtual host at all times.

  **Important**: this command is **very destructive** and should be used with caution. Always test with `--dry-run` first.

* `vhosts enable_deletion_protection` and `vhosts disable_deletion_protection` are two new commands
   for managing [virtual host deletion protection](https://www.rabbitmq.com/docs/vhosts#deletion-protection):

  ```shell
  # Enable deletion protection for a virtual host
  rabbitmqadmin vhosts enable_deletion_protection --name "production-vhost"

  # Disable deletion protection for a virtual host
  rabbitmqadmin vhosts disable_deletion_protection --name "production-vhost"
  ```

  Protected virtual hosts cannot be deleted, either individually using `vhosts delete` or
  as part of bulk operations using `vhosts delete_multiple`. To delete a protected
  virtual host, its protection must be lifted first.

## v2.13.0 (Sep 26, 2025)

### Enhancements

* Memory breakdown commands (`show memory_breakdown_in_bytes` and `show memory_breakdown_in_percent`) now gracefully handle
  cases where memory breakdown stats are not yet available on the target node

* `shovel enable_tls_peer_verification_for_all_source_uris` is a new command that enables TLS peer verification
  for all shovel source URIs:

  ```shell
  # The certificate and private key paths below refer
  # to the files deployed to the target RabbitMQ node(s), not to the
  # local files.
  #
  # As such, these arguments are command-specific and should not be confused
  # with the global `--tls-ca-cert-file`, `--tls-cert-file`, and `--tls-key-file`
  # arguments that are used by `rabbitmqadmin` itself to connect to the target node
  # over the HTTP API.
  rabbitmqadmin shovels enable_tls_peer_verification_for_all_source_uris \
      --node-local-ca-certificate-bundle-path /path/to/node/local/ca_bundle.pem \
      --node-local-client-certificate-file-path /path/to/node/local/client_certificate.pem \
      --node-local-client-private-key-file-path /path/to/node/local/client_private_key.pem
  ```

  See [TLS guide](https://www.rabbitmq.com/docs/ssl#peer-verification) and [Shovel guide](https://www.rabbitmq.com/docs/shovel#tls) to learn more.

* `shovel enable_tls_peer_verification_for_all_destination_uris` is a new command that enables TLS peer verification
  for all shovel destination URIs:

  ```shell
  # Ditto, the certificate and private key paths below refer
  # to the files deployed to the target RabbitMQ node(s), not to the
  # local files.
  rabbitmqadmin shovels enable_tls_peer_verification_for_all_destination_uris \
      --node-local-ca-certificate-bundle-path /path/to/node/local/ca_bundle.pem \
      --node-local-client-certificate-file-path /path/to/node/local/client_certificate.pem \
      --node-local-client-private-key-file-path /path/to/node/local/client_private_key.pem
  ```

### Upgrades

* RabbitMQ HTTP API client was upgraded to [`0.59.0`](https://github.com/michaelklishin/rabbitmq-http-api-rs/releases/tag/v0.59.0)


## v2.12.0 (Sep 23, 2025)

### Enhancements

* `federation enable_tls_peer_verification_for_all_upstreams` is a new command that enables TLS peer verification
  for all federation upstreams:

  ```shell
  # Note that the certificate and private key paths below refer
  # to the files deployed to the target RabbitMQ node(s), not to the
  # local files.
  #
  # As such, these arguments are command-specific and should not be confused
  # with the global `--tls-ca-cert-file`, `--tls-cert-file`, and `--tls-key-file`
  # arguments that are used by `rabbitmqadmin` itself to connect to the target node
  # over the HTTP API.
  rabbitmqadmin federation enable_tls_peer_verification_for_all_upstreams \
      --node-local-ca-certificate-bundle-path /path/to/node/local/ca_bundle.pem \
      --node-local-client-certificate-file-path /path/to/node/local/client_certificate.pem \
      --node-local-client-private-key-file-path /path/to/node/local/client_private_key.pem
  ```

   See [TLS guide](https://www.rabbitmq.com/docs/ssl#peer-verification) and [Federation guide](https://www.rabbitmq.com/docs/federation#tls-connections) to learn more.

 * `shovel disable_tls_peer_verification_for_all_source_uris` is a new command that disables TLS peer verification
   for all shovel source URIs.

   **Important**: this command should **only** be used to undo incorrect shovel source URIs, after a bad deployment, for example,
   if [peer verification](https://www.rabbitmq.com/docs/ssl#peer-verification) was enabled before certificates and keys were
   deployed.

  * `shovel disable_tls_peer_verification_for_all_source_uris` is a new command that disables TLS peer verification
    for all shovel source URIs.

    **Important**: this command should **only** be used to undo incorrect shovel destination URIs (see above).

* All `delete_*` and `clear_*` commands now support the `--idempotently` flag (previously it was just a few):
  - `bindings delete`
  - `close connection`
  - `close user_connections`
  - `connections close`
  - `connections close_of_user`
  - `exchanges delete`
  - `exchanges unbind`
  - `federation delete_upstream`
  - `global_parameters clear`
  - `operator_policies delete`
  - `parameters clear`
  - `policies delete`
  - `queues delete`
  - `shovels delete`
  - `streams delete`
  - `users delete`
  - `vhosts delete`

* Updated `delete_binding` to use the new `BindingDeletionParams` struct API

## v2.11.0 (Sep 22, 2025)

### Enhancements

* `federation disable_tls_peer_verification_for_all_upstreams` is a new command that disables TLS peer verification
  for all federation upstreams.

  **Important**: this command should **only** be used to correct federation upstream URI after a bad deployment, for example,
  if [peer verification](https://www.rabbitmq.com/docs/ssl#peer-verification) was enabled before certificates and keys were
  deployed.

### Upgrades

* RabbitMQ HTTP API client was upgraded to [`0.57.0`](https://github.com/michaelklishin/rabbitmq-http-api-rs/releases/tag/v0.57.0)



## v2.10.0 (Sep 18, 2025)

### Enhancements

* `definitions export_from_vhost` now supports `--transformations`:

  ```shell
  # previously only 'definitions export' supported --transformations
  rabbitmqadmin --vhost "my-vhost" definitions export_from_vhost \
                --transformations prepare_for_quorum_queue_migration,drop_empty_policies \
                --file "my-vhost.definitions.json"
  ```

### Bug Fixes

 * The `prepare_for_quorum_queue_migration` transformation did not remove CMQ-related keys
   such as `x-ha-mode` from [optional queue arguments](https://www.rabbitmq.com/docs/queues#optional-arguments)

### Upgrades

* RabbitMQ HTTP API client was upgraded to [`0.52.0`](https://github.com/michaelklishin/rabbitmq-http-api-rs/releases/tag/v0.52.0)


## v2.9.0 (Aug 25, 2025)

### Enhancements

 * RabbitMQ 4.2 forward compatibility: `shovels list_all` and `shovels list` now can render
   [local shovel](https://github.com/rabbitmq/rabbitmq-server/pull/14256) rows

### Upgrades

* RabbitMQ HTTP API client was upgraded to [`0.44.0`](https://github.com/michaelklishin/rabbitmq-http-api-rs/releases/tag/v0.44.0)


## v2.8.2 (Aug 19, 2025)

### Enhancements

 * `definitions export` is now compatible with RabbitMQ 3.10.0, a series that has
   reached end of life (EOL) in late 2023

### Upgrades

 * RabbitMQ HTTP API client was upgraded to [`0.43.0`](https://github.com/michaelklishin/rabbitmq-http-api-rs/releases/tag/v0.43.0)


## v2.8.1 (Aug 14, 2025)

### Bug Fixes

 * `shovels list` and `shovels list_all` panicked when target cluster had at least one
   static shovel

### Upgrades

 * RabbitMQ HTTP API client was upgraded to [`0.42.0`](https://github.com/michaelklishin/rabbitmq-http-api-rs/releases/tag/v0.42.0)


## v2.8.0 (Aug 11, 2025)

### Bug Fixes

 * `shovels list_all` panicked when one of the shovels was in the `terminated` state

### Enhancements

 * `shovels list` is a new command that lists shovels in a particular virtual host

### Upgrades

* RabbitMQ HTTP API client was upgraded to [`0.41.0`](https://github.com/michaelklishin/rabbitmq-http-api-rs/releases/tag/v0.41.0)


## v2.7.2 (Aug 6, 2025)

### Bug Fixes

 * `shovels declare_amqp091` panicked when the `--source-exchange` argument was not provided,
   even if `--source-queue` was


## v2.7.1 (Jul 17, 2025)

### Bug Fixes

 * Improved handling of missing or impossible to load/parse `--tls-ca-cert-file` on the command line.

   The tool now properly handles cases where a [CA certificate](https://www.rabbitmq.com/docs/ssl#peer-verification) file path is not provided, making
   CA certificate loading optional rather than required, which prevents crashes when TLS is used
   without a custom CA certificate bundle

 * `show overview` could panic when run against a freshly booted RabbitMQ node that did not have certain
   metrics/rates initialized and available. Now those metrics will use the default values for their types,
   such as `0` and `0.0` for the counters, gauges, rates

### Upgrades

* RabbitMQ HTTP API client was upgraded to [`0.40.0`](https://github.com/michaelklishin/rabbitmq-http-api-rs/releases/tag/v0.40.0)


## v2.7.0 (Jul 15, 2025)

### Enhancements

 * `rabbitmqadmin.conf` now supports more TLS-related settings: `ca_certificate_bundle_path` (corresponds to `--tls-ca-cert-file` on the command line),
   `client_certificate_file_path` (corresponds to `--tls-cert-file`), and `client_private_key_file_path` (corresponds to `--tls-key-file`).

   As the names suggest, they are used to configure the CA certificate bundle file path, the client certificate file path,
   and the client private key file path, respectively:

   ```toml
    [production]
    hostname = "(redacted)"
    port = 15671
    username = "user-efe1f4d763f6"
    password = "(redacted)"
    tls = true
    ca_certificate_bundle_path = "/path/to/ca_certificate.pem"
    client_certificate_file_path = "/path/to/client_certificate.pem"
    client_private_key_file_path = "/path/to/client_key.pem"
   ```

   To learn more, see [RabbitMQ's TLS guide](https://www.rabbitmq.com/docs/ssl).

### Bug Fixes

 * Tool version was unintentionally missing from `-h` output (but present in its long counterpart, `--help`)
 * The `tls` setting in `rabbitmqadmin.conf`, a `--use-tls` equivalent, was not respected when connecting to a node
   in certain cases

## v2.6.0 (Jul 12, 2025)

### Enhancements

 * New command, `passwords salt_and_hash`, that implements the [password salting and hashing algorithm](https://www.rabbitmq.com/docs/passwords#computing-password-hash)
   used by RabbitMQ's internal authentication backend:

   ```shell
   rabbitmqadmin passwords salt_and_hash "sEkr37^va1ue"
   # => ┌───────────────┬──────────────────────────────────────────────────┐
   # => │ Result                                                           │
   # => ├───────────────┼──────────────────────────────────────────────────┤
   # => │ key           │ value                                            │
   # => ├───────────────┼──────────────────────────────────────────────────┤
   # => │ password hash │ vRZC0bF0Ut4+6pmcQRSu87S/wRXdHRalgY5DV/5KDd5SzK69 │
   # => └───────────────┴──────────────────────────────────────────────────┘
   ```

   This value can be passed as a `--password-hash` when creating a user with the `users declare`
   command.

 * `users declare` now supports a new argument, `--hashing-algorithm`, that accepts two
   possible values: `sha256` (the default) and `sha512`:

   ```shell
   # RabbitMQ nodes must also be configured to use SHA-512 password hashing,
   # or this user won't be able to authenticate against them
   rabbitmqadmin users declare --username "username43742" --password "example_%^4@8s7" --hashing-algorithm "sha512"
   ```

   Target RabbitMQ nodes must be [configured](https://www.rabbitmq.com/docs/passwords#changing-algorithm) to use the same hashing algorithm (SHA-256 is
   used by default).


## v2.5.0 (Jul 11, 2025)

### Enhancements

 * `definitions export` now supports a new transformation: `prepare_for_quorum_queue_migration`.

   ```shell
   rabbitmqadmin definitions export --transformations prepare_for_quorum_queue_migration,drop_empty_policies --stdout
   ```

   This one not only strips off the CMQ-related keys
   but also handles an incompatible `"overflow"`/`"x-overflow"` key value
   and `"queue-mode"`/`"x-queue-mode"` keys, both not supported
   by quorum queues.

### Bug Fixes

 * `export definitions` CLI interface was unintentionally different from that of `definitions export`.
    Note that `export definitions` only exists for better backwards compatibility with `rabbitmqadmin` v1,
    use `definitions export` when possible.


## v2.4.0 (Jul 4, 2025)

### Bug Fixes

 * `connections list` failed to deserialize a list of connections that included direct connections
   (as in the Erlang AMQP 0-9-1 client), namely local connections of shovels and federation links.

   GitHub issue: [#68](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/68)

### Upgrades

 * RabbitMQ HTTP API client was upgraded to [`0.36.0`](https://github.com/michaelklishin/rabbitmq-http-api-rs/releases/tag/v0.36.0)


## v2.3.0 (Jun 30, 2025)

 * RabbitMQ HTTP API client was upgraded to [`0.35.0`](https://github.com/michaelklishin/rabbitmq-http-api-rs/releases/tag/v0.35.0) to fix a `connections list` command
   panic.


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
