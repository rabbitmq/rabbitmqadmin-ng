// Copyright (C) 2023-2025 RabbitMQ Core Team (teamrabbitmq@gmail.com)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use std::path::PathBuf;

use super::constants::*;
use super::static_urls::*;
use super::tanzu_cli::tanzu_subcommands;
use crate::output::TableStyle;
use clap::{Arg, ArgAction, ArgGroup, Command, value_parser};
use rabbitmq_http_client::commons::{
    BindingDestinationType, ExchangeType, PolicyTarget, QueueType, ShovelAcknowledgementMode,
    SupportedProtocol,
};

pub fn parser() -> Command {
    let after_help = color_print::cformat!(
        r#"
<bold><blue>Documentation and Community Resources</blue></bold>

  RabbitMQ docs: {}
  GitHub Discussions: {}
  Discord server: {}

<bold><blue>Contribute</blue></bold>

  On GitHub: {}"#,
        RABBITMQ_DOC_GUIDES_URL,
        GITHUB_DISCUSSIONS_URL,
        DISCORD_SERVER_INVITATION_URL,
        GITHUB_REPOSITORY_URL
    );

    Command::new("rabbitmqadmin")
        .version(clap::crate_version!())
        .author("RabbitMQ Core Team")
        .about("rabbitmqadmin gen 2")
        .long_about(format!(
            "RabbitMQ CLI that uses the HTTP API. Version: {}",
            clap::crate_version!()
        ))
        .after_help(after_help)
        .disable_version_flag(true)
        // --config-file
        .arg(
            Arg::new("config_file_path")
                .short('c')
                .long("config")
                .value_parser(value_parser!(PathBuf))
                .default_value(DEFAULT_CONFIG_FILE_PATH),
        )
        // --node
        // This is NOT the same as --node in case of rabbitmqctl, rabbitmq-diagnostics, etc.
        // This is node section name in the configuration file. MK.
        .arg(
            Arg::new("node_alias")
                .short('N')
                .long("node")
                .required(false)
                .default_value(DEFAULT_NODE_ALIAS),
        )
        // --host
        .arg(
            Arg::new("host")
                .short('H')
                .long("host")
                .help("HTTP API hostname to use when connecting"),
        )
        .visible_alias("hostname")
        // --port
        .arg(
            Arg::new("port")
                .short('P')
                .long("port")
                .help("HTTP API port to use when connecting")
                .required(false)
                .value_parser(value_parser!(u16))
                .default_value(DEFAULT_PORT_STR),
        )
        // --base-uri
        .arg(
            Arg::new("base_uri")
                .short('U')
                .long("base-uri")
                .help("base HTTP API endpoint URI")
                .required(false)
                .conflicts_with_all(["host", "port"]),
        )
        // --path-prefix
        .arg(
            Arg::new("path_prefix")
                .long("path-prefix")
                .help("use if target node uses a path prefix. Defaults to '/api'"),
        )
        // --vhost
        .arg(
            Arg::new("vhost")
                .short('V')
                .long("vhost")
                .help("target virtual host. Defaults to '/'"),
        )
        // --username
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .help(format!(
                    "this user must have the permissions for HTTP API access, see {}",
                    HTTP_API_ACCESS_PERMISSIONS_GUIDE_URL
                )),
        )
        // --password
        .arg(
            Arg::new("password")
                .short('p')
                .long("password")
                .requires("username")
                .help("requires username to be specified via --username or in the config file"),
        )
        // --insecure
        .arg(
            Arg::new("insecure")
                .short('k')
                .long("insecure")
                .required(false)
                .help("disables TLS peer (certificate chain) verification")
                .value_parser(value_parser!(bool))
                .action(ArgAction::SetTrue),
        )
        // --tls
        .arg(
            Arg::new("tls")
                .long("use-tls")
                .help("use TLS (HTTPS) for HTTP API requests ")
                .value_parser(value_parser!(bool))
                .action(ArgAction::SetTrue)
                .requires("tls-ca-cert-file"),
        )
        // --tls-ca-cert-file
        .arg(
            Arg::new("tls-ca-cert-file")
                .long("tls-ca-cert-file")
                .required(false)
                .requires("tls")
                .help("Local path to a CA certificate file in the PEM format")
                .value_parser(value_parser!(PathBuf)),
        )
        // --tls-cert-file
        .arg(
            Arg::new("tls-cert-file")
                .long("tls-cert-file")
                .required(false)
                .requires("tls-key-file")
                .help("Local path to a client certificate file in the PEM format")
                .value_parser(value_parser!(PathBuf)),
        )
        // --tls-key-file
        .arg(
            Arg::new("tls-key-file")
                .long("tls-key-file")
                .required(false)
                .requires("tls-cert-file")
                .help("Local path to a client private key file in the PEM format")
                .value_parser(value_parser!(PathBuf)),
        )
        // --quiet
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("produce less output")
                .required(false)
                .value_parser(value_parser!(bool))
                .action(ArgAction::SetTrue),
        )
        // --non-interactive
        .arg(
            Arg::new("non_interactive")
                .long("non-interactive")
                .help("pass when invoking from scripts")
                .conflicts_with("table_style")
                .required(false)
                .value_parser(value_parser!(bool))
                .action(ArgAction::SetTrue),
        )
        // --table-style
        .arg(
            Arg::new("table_style")
                .long("table-style")
                .help("style preset to apply to output tables: modern, borderless, ascii, dots, psql, markdown, sharp")
                .conflicts_with("non_interactive")
                .required(false)
                .value_parser(value_parser!(TableStyle))
        )
        .subcommand_required(true)
        .subcommand_value_name("command")
        .subcommands([
            Command::new("show")
                .about("overview")
                .after_long_help(color_print::cformat!(
                    "<bold>Doc guide</bold>: {}",
                    MONITORING_GUIDE_URL
                ))
                .subcommand_value_name("summary")
                .subcommands(show_subcommands()),
            Command::new("list")
                .about("lists objects by type")
                .subcommand_value_name("objects")
                .subcommands(list_subcommands()),
            Command::new("declare")
                .about("creates or declares things")
                .subcommand_value_name("object")
                .subcommands(declare_subcommands()),
            Command::new("delete")
                .about("deletes objects")
                .subcommand_value_name("object")
                .subcommands(delete_subcommands()),
            Command::new("purge")
                .about("purges queues")
                .subcommand_value_name("queue")
                .subcommands(purge_subcommands()),
            Command::new("policies")
                .about("operations on policies")
                .subcommand_value_name("policy")
                .subcommands(policies_subcommands()),
            Command::new("health_check")
                .about("runs health checks")
                .subcommand_value_name("check")
                .subcommands(health_check_subcommands())
                .after_long_help(color_print::cformat!(
                    r#"<bold>Doc guides</bold>:

 * {}
 * {}"#,
                    HEALTH_CHECK_GUIDE_URL,
                    DEPRECATED_FEATURE_GUIDE_URL
                )),
            Command::new("close")
                .about("closes connections")
                .subcommand_value_name("connection")
                .subcommands(close_subcommands()),
            Command::new("rebalance")
                .about("rebalances queue leaders")
                .after_long_help(color_print::cformat!(
                    "<bold>Doc guide</bold>: {}",
                    QUORUM_QUEUE_GUIDE_URL
                ))
                .subcommand_value_name("queues")
                .subcommands(rebalance_subcommands()),
            Command::new("definitions")
                .about("operations on definitions (everything except for messages: virtual hosts, queues, streams, exchanges, bindings, users, etc)")
                .after_long_help(color_print::cformat!(
                    "<bold>Doc guide</bold>: {}",
                    DEFINITION_GUIDE_URL
                ))
                .subcommand_value_name("export")
                .subcommands(definitions_subcommands()),
            Command::new("export")
                .about("see 'definitions export'")
                .after_long_help(color_print::cformat!(
                    "<bold>Doc guide</bold>: {}",
                    DEFINITION_GUIDE_URL
                ))
                .subcommand_value_name("definitions")
                .subcommands(export_subcommands()),
            Command::new("import")
                .about("see 'definitions import'")
                .after_long_help(color_print::cformat!(
                    "<bold>Doc guide</bold>: {}",
                    DEFINITION_GUIDE_URL
                ))
                .subcommand_value_name("definitions")
                .subcommands(import_subcommands()),
            Command::new("feature_flags")
                .about("operations on feature flags")
                .after_long_help(color_print::cformat!(
                    "<bold>Doc guide</bold>: {}",
                    FEATURE_FLAG_GUIDE_URL
                ))
                .subcommand_value_name("feature flag")
                .subcommands(feature_flags_subcommands()),
            Command::new("deprecated_features")
                .about("operations on deprecated features")
                .after_long_help(color_print::cformat!(
                    "<bold>Doc guide</bold>: {}",
                    DEPRECATED_FEATURE_GUIDE_URL
                ))
                .subcommand_value_name("deprecated feature")
                .subcommands(deprecated_features_subcommands()),
            Command::new("publish")
                .about(color_print::cstr!("publishes (<red>inefficiently</red>) message(s) to a queue or a stream. <bold><red>Only suitable for development and test environments</red></bold>."))
                .after_long_help(color_print::cformat!("<bold>Doc guide</bold>: {}", PUBLISHER_GUIDE_URL))
                .subcommand_value_name("message")
                .subcommands(publish_subcommands()),
            Command::new("get")
                .about(color_print::cstr!("fetches message(s) from a queue or stream via <bold><red>polling</red></bold>. <bold><red>Only suitable for development and test environments</red></bold>."))
                .after_long_help(color_print::cformat!("<bold>Doc guide</bold>: {}", POLLING_CONSUMER_GUIDE_URL))
                .subcommand_value_name("message")
                .subcommands(get_subcommands()),
            Command::new("shovels")
                .about("Operations on shovels")
                .after_long_help(color_print::cformat!("<bold>Doc guide</bold>: {}", SHOVEL_GUIDE_URL))
                .subcommand_value_name("shovels")
                .subcommands(shovel_subcommands()),
            Command::new("tanzu")
                .about("Tanzu RabbitMQ-specific commands")
                // TODO: documentation link
                .subcommand_value_name("subcommand")
                .subcommands(tanzu_subcommands()),
        ])
}

fn list_subcommands() -> [Command; 19] {
    [
        Command::new("nodes").long_about("Lists cluster members"),
        Command::new("users").long_about("Lists users in the internal database"),
        Command::new("vhosts")
            .long_about("Lists virtual hosts")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                VIRTUAL_HOST_GUIDE_URL
            )),
        Command::new("permissions")
            .long_about("Lists user permissions")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                ACCESS_CONTROL_GUIDE_URL
            )),
        Command::new("connections")
            .long_about("Lists client connections")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                CONNECTION_GUIDE_URL
            )),
        Command::new("user_connections")
            .arg(
                Arg::new("username")
                    .short('u')
                    .long("username")
                    .required(true)
                    .help("Name of the user whose connections to list"),
            )
            .long_about("Lists client connections that authenticated with a specific username")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                CONNECTION_GUIDE_URL
            )),
        Command::new("channels")
            .long_about("Lists AMQP 0-9-1 channels")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                CHANNEL_GUIDE_URL
            )),
        Command::new("queues")
            .long_about("Lists queues and streams")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                QUEUE_GUIDE_URL
            )),
        Command::new("exchanges").long_about("Lists exchanges"),
        Command::new("bindings").long_about("Lists bindings"),
        Command::new("consumers")
            .long_about("Lists consumers")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                CONSUMER_GUIDE_URL
            )),
        Command::new("parameters")
            .arg(
                Arg::new("component")
                    .long("component")
                    .help("component (for example: federation-upstream, vhost-limits)")
                    .required(false),
            )
            .long_about("Lists runtime parameters")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                RUNTIME_PARAMETER_GUIDE_URL
            )),
        Command::new("policies")
            .long_about("Lists policies")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                POLICY_GUIDE_URL
            )),
        Command::new("operator_policies")
            .long_about("Lists operator policies")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                OPERATOR_POLICY_GUIDE_URL
            )),
        Command::new("vhost_limits")
            .long_about("Lists virtual host (resource) limits")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                VIRTUAL_HOST_GUIDE_URL
            )),
        Command::new("user_limits")
            .arg(
                Arg::new("user")
                    .long("user")
                    .help("username")
                    .required(false),
            )
            .long_about("Lists per-user (resource) limits")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                USER_LIMIT_GUIDE_URL
            )),
        Command::new("feature_flags")
            .long_about("Lists feature flags and their cluster state")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                FEATURE_FLAG_GUIDE_URL
            )),
        Command::new("deprecated_features")
            .long_about("Lists all deprecated features")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                DEPRECATED_FEATURE_GUIDE_URL
            )),
        Command::new("deprecated_features_in_use")
            .long_about("Lists the deprecated features that are in used in the cluster")
            .after_long_help(color_print::cformat!(
                "<bold>Doc guide</bold>: {}",
                DEPRECATED_FEATURE_GUIDE_URL
            )),
    ]
}

fn declare_subcommands() -> [Command; 12] {
    [
        Command::new("user")
            .about("creates a user")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("username")
                    .required(true),
            )
            .arg(
                Arg::new("password_hash")
                    .help(color_print::cformat!("salted password hash, see {}", PASSWORD_GUIDE_URL))
                    .long("password-hash")
                    .required(false)
                    .default_value(""),
            )
            .arg(
                Arg::new("password")
                    .long("password")
                    .help(color_print::cformat!("prefer providing a hash, see {}", PASSWORD_GUIDE_URL))
                    .required(false)
                    .default_value(""),
            )
            .arg(
                Arg::new("tags")
                    .long("tags")
                    .help("a list of comma-separated tags")
                    .default_value(""),
            ),
        Command::new("vhost")
            .about("creates a virtual host")
            .after_long_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", VIRTUAL_HOST_GUIDE_URL))
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("virtual host name")
                    .required(true),
            )
            .arg(
                Arg::new("default_queue_type")
                    .long("default-queue-type")
                    .required(false)
                    .default_value(DEFAULT_QUEUE_TYPE)
                    .help(color_print::cformat!("default queue type, one of: <bold>classic</bold>, <bright-blue>quorum</bright-blue>, <bright-magenta>stream</bright-magenta>"))
            )
            .arg(
                Arg::new("description")
                    .long("description")
                    .required(false)
                    .help("what's the purpose of this virtual host?"),
            )
            .arg(
                Arg::new("tracing")
                    .long("tracing")
                    .required(false)
                    .action(ArgAction::SetTrue)
                    .help("should tracing be enabled for this virtual host?"),
            ),
        Command::new("permissions")
            .about("grants permissions to a user")
            .after_long_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", ACCESS_CONTROL_GUIDE_URL))
            .arg(
                Arg::new("user")
                    .long("user")
                    .help("username")
                    .required(true),
            )
            .arg(
                Arg::new("configure")
                    .long("configure")
                    .help("name pattern for configuration access")
                    .required(true),
            )
            .arg(
                Arg::new("read")
                    .long("read")
                    .help("name pattern for read access")
                    .required(true),
            )
            .arg(
                Arg::new("write")
                    .long("write")
                    .help("name pattern for write access")
                    .required(true),
            ),
        Command::new("queue")
            .about("declares a queue or a stream")
            .after_long_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", QUEUE_GUIDE_URL))
            .arg(Arg::new("name").long("name").required(true).help("name"))
            .arg(
                Arg::new("type")
                    .long("type")
                    .help("queue type")
                    .value_parser(value_parser!(QueueType))
                    .required(false)
                    .default_value("classic"),
            )
            .arg(
                Arg::new("durable")
                    .long("durable")
                    .help("should it persist after a restart")
                    .required(false)
                    .value_parser(value_parser!(bool)),
            )
            .arg(
                Arg::new("auto_delete")
                    .long("auto-delete")
                    .help("should it be deleted when the last consumer disconnects")
                    .required(false)
                    .value_parser(value_parser!(bool)),
            )
            .arg(
                Arg::new("arguments")
                    .long("arguments")
                    .help("additional exchange arguments")
                    .required(false)
                    .default_value("{}")
                    .value_parser(value_parser!(String)),
            ),
        Command::new("stream")
            .about("declares a stream")
            .after_long_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", STREAM_GUIDE_URL))
            .arg(Arg::new("name").long("name").required(true).help("name"))
            .arg(
                Arg::new("expiration")
                    .long("expiration")
                    .help("stream expiration, e.g. 12h for 12 hours, 7D for 7 days, or 1M for 1 month")
                    .required(true)
                    .value_parser(value_parser!(String)),
            )
            .arg(
                Arg::new("max_length_bytes")
                    .long("max-length-bytes")
                    .help("maximum stream length in bytes")
                    .required(false)
                    .value_parser(value_parser!(u64)),
            )
            .arg(
                Arg::new("max_segment_length_bytes")
                    .long("stream-max-segment-size-bytes")
                    .help("maximum stream segment file length in bytes")
                    .required(false)
                    .value_parser(value_parser!(u64)),
            )
            .arg(
                Arg::new("arguments")
                    .long("arguments")
                    .help("additional exchange arguments")
                    .required(false)
                    .default_value("{}")
                    .value_parser(value_parser!(String)),
            ),
        Command::new("exchange")
            .about("declares an exchange")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("exchange name")
                    .required(true),
            )
            .arg(
                Arg::new("type")
                    .long("type")
                    .help("exchange type")
                    .value_parser(value_parser!(ExchangeType))
                    .required(false),
            )
            .arg(
                Arg::new("durable")
                    .long("durable")
                    .help("should it persist after a restart")
                    .required(false)
                    .value_parser(value_parser!(bool)),
            )
            .arg(
                Arg::new("auto_delete")
                    .long("auto-delete")
                    .help("should it be deleted when the last queue is unbound")
                    .required(false)
                    .value_parser(value_parser!(bool)),
            )
            .arg(
                Arg::new("arguments")
                    .long("arguments")
                    .help("additional exchange arguments")
                    .required(false)
                    .default_value("{}")
                    .value_parser(value_parser!(String)),
            ),
        Command::new("binding")
            .about("creates a binding between a source exchange and a destination (a queue or an exchange)")
            .arg(
                Arg::new("source")
                    .long("source")
                    .help("source exchange")
                    .required(true),
            )
            .arg(
                Arg::new("destination_type")
                    .long("destination-type")
                    .help("destination type: exchange or queue")
                    .required(true)
                    .value_parser(value_parser!(BindingDestinationType)),
            )
            .arg(
                Arg::new("destination")
                    .long("destination")
                    .help("destination exchange/queue name")
                    .required(true),
            )
            .arg(
                Arg::new("routing_key")
                    .long("routing-key")
                    .help("routing key")
                    .required(true),
            )
            .arg(
                Arg::new("arguments")
                    .long("arguments")
                    .help("additional arguments")
                    .required(false)
                    .default_value("{}")
                    .value_parser(value_parser!(String)),
            ),
        Command::new("parameter").
            about("sets a runtime parameter")
            .after_long_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", RUNTIME_PARAMETER_GUIDE_URL))
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("parameter's name")
                    .required(true)
            ).arg(
            Arg::new("component")
                .long("component")
                .help("component (eg. federation)")
                .required(true))
            .arg(
                Arg::new("value")
                    .long("value")
                    .help("parameter's value")
                    .required(true)),
        Command::new("policy")
            .about("creates or updates a policy")
            .after_long_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", POLICY_GUIDE_URL))
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("policy name")
                    .required(true),
            )
            .arg(
                Arg::new("pattern")
                    .long("pattern")
                    .help("the pattern that is used to match entity (queue, stream, exchange) names")
                    .required(true),
            )
            .arg(
                Arg::new("apply_to")
                    .long("apply-to")
                    .help("entities to apply to (queues, classic_queues, quorum_queues, streams, exchanges, all)")
                    .value_parser(value_parser!(PolicyTarget))
                    .required(true),
            )
            .arg(
                Arg::new("priority")
                    .long("priority")
                    .help("policy priority (only the policy with the highest priority is effective)")
                    .required(false)
                    .default_value("0"),
            )
            .arg(
                Arg::new("definition")
                    .long("definition")
                    .help("policy definition")
                    .required(true),
            ),
        Command::new("operator_policy")
            .about("creates or updates an operator policy")
            .after_long_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", OPERATOR_POLICY_GUIDE_URL))
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("operator policy name")
                    .required(true),
            )
            .arg(
                Arg::new("pattern")
                    .long("pattern")
                    .help("queue/exchange name pattern")
                    .required(true),
            )
            .arg(
                Arg::new("apply_to")
                    .long("apply-to")
                    .help("entities to apply to (queues, classic_queues, quorum_queues, streams, exchanges, all)")
                    .value_parser(value_parser!(PolicyTarget))
                    .required(true),
            )
            .arg(
                Arg::new("priority")
                    .long("priority")
                    .help("policy priority (only the policy with the highest priority is effective)")
                    .required(false)
                    .default_value("0"),
            )
            .arg(
                Arg::new("definition")
                    .long("definition")
                    .help("policy definition")
                    .required(true),
            ),
        Command::new("vhost_limit")
            .about("set a vhost limit")
            .after_long_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", VIRTUAL_HOST_LIMIT_GUIDE_URL))
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("limit name (eg. max-connections, max-queues)")
                    .required(true),
            )
            .arg(
                Arg::new("value")
                    .long("value")
                    .help("limit value")
                    .required(true),
            ),
        Command::new("user_limit")
            .about("set a user limit")
            .after_long_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", USER_LIMIT_GUIDE_URL))
            .arg(
                Arg::new("user")
                    .long("user")
                    .help("username")
                    .required(true),
            )
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("limit name (eg. max-connections, max-queues)")
                    .required(true),
            )
            .arg(
                Arg::new("value")
                    .long("value")
                    .help("limit value")
                    .required(true),
            )
    ]
}

fn show_subcommands() -> [Command; 5] {
    let overview_cmd = Command::new("overview")
        .about("displays a essential information about target node and its cluster");
    let churn_cmd = Command::new("churn").about("displays object churn metrics");
    let endpoint_cmd = Command::new("endpoint")
        .about("for troubleshooting: displays the computed HTTP API endpoint URI");
    let memory_breakdown_in_bytes_cmd = Command::new("memory_breakdown_in_bytes")
        .about("provides a memory footprint breakdown (in bytes) for the target node")
        .arg(
            Arg::new("node")
                .long("node")
                .help("target node, must be a cluster member")
                .required(true),
        )
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            MEMORY_FOOTPRINT_GUIDE_URL
        ));

    let memory_breakdown_in_percent_cmd = Command::new("memory_breakdown_in_percent")
        .about("provides a memory footprint breakdown (in percent) for the target node")
        .arg(
            Arg::new("node")
                .long("node")
                .help("target node, must be a cluster member")
                .required(true),
        )
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            MEMORY_FOOTPRINT_GUIDE_URL
        ));

    [
        overview_cmd,
        churn_cmd,
        endpoint_cmd,
        memory_breakdown_in_bytes_cmd,
        memory_breakdown_in_percent_cmd,
    ]
}

fn delete_subcommands() -> [Command; 13] {
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);

    [
        Command::new("user")
            .about("deletes a user")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("username")
                    .required(true),
            )
            .arg(idempotently_arg.clone()),
        Command::new("vhost")
            .about("deletes a virtual host")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("virtual host")
                    .required(true),
            )
            .arg(idempotently_arg.clone()),
        Command::new("permissions")
            .about("revokes user permissions to a given vhost")
            .arg(
                Arg::new("user")
                    .long("user")
                    .help("username")
                    .required(true),
            )
            .arg(idempotently_arg.clone()),
        Command::new("queue")
            .about("deletes a queue")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("queue name")
                    .required(true),
            )
            .arg(idempotently_arg.clone()),
        Command::new("stream")
            .about("deletes a stream")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("stream name")
                    .required(true),
            )
            .arg(idempotently_arg.clone()),
        Command::new("exchange")
            .about("deletes an exchange")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("exchange name")
                    .required(true),
            )
            .arg(idempotently_arg.clone()),
        Command::new("binding")
            .about("deletes a binding")
            .arg(
                Arg::new("source")
                    .long("source")
                    .help("source exchange")
                    .required(true),
            )
            .arg(
                Arg::new("destination_type")
                    .long("destination-type")
                    .help("destination type: exchange or queue")
                    .required(true),
            )
            .arg(
                Arg::new("destination")
                    .long("destination")
                    .help("destination exchange/queue name")
                    .required(true),
            )
            .arg(
                Arg::new("routing_key")
                    .long("routing-key")
                    .help("routing key")
                    .required(true),
            )
            .arg(
                Arg::new("arguments")
                    .long("arguments")
                    .help("additional arguments")
                    .required(false)
                    .default_value("{}")
                    .value_parser(value_parser!(String)),
            ),
        Command::new("parameter")
            .about("clears a runtime parameter")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("parameter's name")
                    .required(true),
            )
            .arg(
                Arg::new("component")
                    .long("component")
                    .help("component (eg. federation-upstream)")
                    .required(true),
            ),
        Command::new("policy").about("deletes a policy").arg(
            Arg::new("name")
                .long("name")
                .help("policy name")
                .required(true),
        ),
        Command::new("operator_policy")
            .about("deletes an operator policy")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("operator policy name")
                    .required(true),
            ),
        Command::new("vhost_limit")
            .about("delete a vhost limit")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("limit name (eg. max-connections, max-queues)")
                    .required(true),
            ),
        Command::new("user_limit")
            .about("clears a user limit")
            .arg(
                Arg::new("user")
                    .long("user")
                    .help("username")
                    .required(true),
            )
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("limit name (eg. max-connections, max-queues)")
                    .required(true),
            ),
        Command::new("shovel")
            .about("delete a shovel")
            .arg(idempotently_arg.clone())
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("shovel name")
                    .required(true),
            ),
    ]
}

fn purge_subcommands() -> [Command; 1] {
    [Command::new("queue")
        .long_about("purges (permanently removes unacknowledged messages from) a queue")
        .arg(
            Arg::new("name")
                .long("name")
                .help("name of the queue to purge")
                .required(true),
        )]
}

fn policies_subcommands() -> [Command; 5] {
    let declare_cmd = Command::new("declare")
        .about("creates or updates a policy")
        .after_long_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", POLICY_GUIDE_URL))
        .arg(
            Arg::new("name")
                .long("name")
                .help("policy name")
                .required(true),
        )
        .arg(
            Arg::new("pattern")
                .long("pattern")
                .help("the pattern that is used to match entity (queue, stream, exchange) names")
                .required(true),
        )
        .arg(
            Arg::new("apply_to")
                .long("apply-to")
                .help("entities to apply to (queues, classic_queues, quorum_queues, streams, exchanges, all)")
                .value_parser(value_parser!(PolicyTarget))
                .required(true),
        )
        .arg(
            Arg::new("priority")
                .long("priority")
                .help("policy priority (only the policy with the highest priority is effective)")
                .required(false)
                .default_value("0"),
        )
        .arg(
            Arg::new("definition")
                .long("definition")
                .help("policy definition")
                .required(true),
        );

    let list_cmd = Command::new("list")
        .long_about("lists policies")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            POLICY_GUIDE_URL
        ));

    let delete_cmd = Command::new("delete").about("deletes a policy").arg(
        Arg::new("name")
            .long("name")
            .help("policy name")
            .required(true),
    );

    let list_in_cmd = Command::new("list_in")
        .about("lists policies in a specific virtual host")
        .arg(
            Arg::new("apply_to")
                .long("apply-to")
                .value_parser(value_parser!(PolicyTarget)),
        );

    let list_matching_cmd = Command::new("list_matching_object")
        .about("lists policies that match an object (queue, stream, exchange) name")
        .arg(
            Arg::new("name")
                .long("name")
                .help("name to verify")
                .required(true),
        )
        .arg(
            Arg::new("type")
                .long("type")
                .value_parser(value_parser!(PolicyTarget))
                .required(true)
                .help("target type, one of 'queues', 'streams', 'exchanges'"),
        );

    [
        declare_cmd,
        list_cmd,
        delete_cmd,
        list_in_cmd,
        list_matching_cmd,
    ]
}

fn health_check_subcommands() -> [Command; 6] {
    let node_is_quorum_critical_after_help = color_print::cformat!(
        r#"
<bold>Doc guides</bold>:

 * {}
 * {}"#,
        QUORUM_QUEUE_FAILURE_HANDLING_GUIDE_URL,
        UPGRADE_GUIDE_URL
    );

    let local_alarms = Command::new("local_alarms")
        .about("checks if there are any resource alarms in effect on the target node");
    let cluster_wide_alarms = Command::new("cluster_wide_alarms")
        .about("checks if there are any resource alarms in effect across the entire cluster");
    let node_is_quorum_critical = Command::new("node_is_quorum_critical")
        .about("fails if there are queues/streams with minimum online quorum (queues/streams that will lose their quorum if the target node shuts down)")
        .after_long_help(node_is_quorum_critical_after_help);
    let deprecated_features_in_use = Command::new("deprecated_features_in_use")
        .about("fails if there are any deprecated features in use in the cluster")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEPRECATED_FEATURE_GUIDE_URL
        ));

    let port_listener = Command::new("port_listener")
        .about(
            "verifies that there's a reachable TCP listener on the given port on the target node",
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_parser(value_parser!(u16)),
        )
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            HEALTH_CHECK_GUIDE_URL
        ));

    let protocol_listener = Command::new("protocol_listener")
        .about(
            "verifies that there's a reachable TCP listener on the given protocol alias on the target node",
        )
        .arg(
            Arg::new("protocol")
                .long("protocol")
                .value_parser(value_parser!(SupportedProtocol))
                .long_help("An alias for one of the protocols that RabbitMQ supports, with or without TLS: 'amqp', 'amqp/ssl', 'stream', 'stream/ssl', 'mqtt', 'mqtt/ssl', 'stomp', 'stomp/ssl', 'http/web-mqtt', 'https/web-mqtt', 'http/web-stomp', 'https/web-stomp', 'http/prometheus', 'https/prometheus', 'http', 'https'"),
        )
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            HEALTH_CHECK_GUIDE_URL
        ));

    [
        local_alarms,
        cluster_wide_alarms,
        node_is_quorum_critical,
        deprecated_features_in_use,
        port_listener,
        protocol_listener,
    ]
}

fn rebalance_subcommands() -> [Command; 1] {
    [Command::new("queues").about("rebalances queue leaders")]
}

fn close_subcommands() -> [Command; 2] {
    let close_connection = Command::new("connection")
        .about("closes a client connection")
        .arg(
            Arg::new("name")
                .long("name")
                .help("connection name (identifying string)")
                .required(true),
        );
    let close_user_connections = Command::new("user_connections")
        .about("closes all connections that authenticated with a specific username")
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .help("Name of the user whose connections to close")
                .required(true),
        );
    [close_connection, close_user_connections]
}

fn definitions_subcommands() -> [Command; 4] {
    let export_cmd = Command::new("export")
        .about("export cluster-wide definitions")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEFINITION_GUIDE_URL
        ))
        .arg(
            Arg::new("file")
                .long("file")
                .help("output file path or '-' for standard output")
                .required(false)
                .default_value("-"),
        )
        .arg(
            Arg::new("transformations")
                .long("transformations")
                .short('t')
                .long_help(
                    r#"
A comma-separated list of names of the definition transformations to apply.

Supported transformations:

 * strip_cmq_keys_from_policies
 * drop_empty_policies
 * exclude_users
 * exclude_permissions
 * exclude_runtime_parameters
 * exclude_policies

Examples:

 * --transformations strip_cmq_keys_from_policies,drop_empty_policies
 * --transformations exclude_users,exclude_permissions
 * --transformations exclude_runtime_parameters,exclude_policies
                "#,
                )
                .num_args(1..)
                .value_delimiter(',')
                .action(ArgAction::Append)
                .required(false),
        );

    let export_from_vhost_cmd = Command::new("export_from_vhost")
        .about("export definitions of a specific virtual host")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEFINITION_GUIDE_URL
        ))
        .arg(
            Arg::new("file")
                .long("file")
                .help("output file path or '-' for standard output")
                .required(false)
                .default_value("-"),
        );

    let import_cmd = Command::new("import")
        .about("import cluster-wide definitions (of multiple virtual hosts)")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEFINITION_GUIDE_URL
        ))
        .arg(
            Arg::new("file")
                .long("file")
                .help("JSON file with cluster-wide definitions")
                .required(true),
        );

    let import_into_vhost_cmd = Command::new("import_into_vhost")
        .about("import a virtual host-specific definitions file into a virtual host")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEFINITION_GUIDE_URL
        ))
        .arg(
            Arg::new("file")
                .long("file")
                .help("JSON file with virtual host-specific definitions")
                .required(true),
        );

    [
        export_cmd,
        export_from_vhost_cmd,
        import_cmd,
        import_into_vhost_cmd,
    ]
}

fn export_subcommands() -> [Command; 1] {
    let definitions = Command::new("definitions")
        .about("prefer 'definitions export'")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEFINITION_GUIDE_URL
        ))
        .arg(
            Arg::new("file")
                .long("file")
                .help("output path")
                .required(false)
                .default_value("-"),
        )
        .arg(
            Arg::new("transformations")
                .long("transformations")
                .short('t')
                .long_help(
                    r#"
A comma-separated list of names of the definition transformations to apply.

Supported transformations:

 * strip_cmq_keys_from_policies
 * drop_empty_policies

Example use: --transformations strip_cmq_keys_from_policies,drop_empty_policies
                "#,
                )
                .num_args(1..)
                .value_delimiter(',')
                .action(ArgAction::Append)
                .required(false),
        );
    [definitions]
}

fn import_subcommands() -> [Command; 1] {
    [Command::new("definitions")
        .about("prefer 'definitions import'")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEFINITION_GUIDE_URL
        ))
        .arg(
            Arg::new("file")
                .long("file")
                .help("JSON file with definitions")
                .required(true),
        )]
}

pub fn feature_flags_subcommands() -> [Command; 3] {
    let list_cmd = Command::new("list")
        .long_about("lists feature flags and their cluster state")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            FEATURE_FLAG_GUIDE_URL
        ));

    let enable_cmd = Command::new("enable")
        .long_about("enables a feature flag")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            FEATURE_FLAG_GUIDE_URL
        ))
        .arg(
            Arg::new("name")
                .long("name")
                .help("feature flag name (identifier)")
                .required(true),
        );

    let enable_all_cmd = Command::new("enable_all")
        .long_about("enables all stable feature flags")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            FEATURE_FLAG_GUIDE_URL
        ));

    [list_cmd, enable_cmd, enable_all_cmd]
}

pub fn deprecated_features_subcommands() -> [Command; 2] {
    let list_cmd = Command::new("list")
        .long_about("lists deprecated features")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEPRECATED_FEATURE_GUIDE_URL
        ));

    let list_in_use_cmd = Command::new("list_used")
        .long_about("lists the deprecated features that are found to be in use in the cluster")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEPRECATED_FEATURE_GUIDE_URL
        ));

    [list_cmd, list_in_use_cmd]
}

pub fn publish_subcommands() -> [Command; 1] {
    [Command::new("message")
        .about("publishes a message to an exchange")
        .about(color_print::cstr!("publishes (<red>inefficiently</red>) message(s) to a queue or a stream. <bold><red>Only suitable for development and test environments</red></bold>. Prefer messaging or streaming protocol clients!"))
        .after_long_help(color_print::cformat!("<bold>Doc guide</bold>: {}", PUBLISHER_GUIDE_URL))
        .arg(
            Arg::new("routing_key")
                .short('k')
                .long("routing-key")
                .required(false)
                .default_value("")
                .help("Name of virtual host"),
        )
        .arg(
            Arg::new("exchange")
                .short('e')
                .long("exchange")
                .required(false)
                .default_value("")
                .help("Exchange name (defaults to empty)"),
        )
        .arg(
            Arg::new("payload")
                .short('m')
                .long("payload")
                .required(false)
                .default_value("test")
                .help("Message payload/body"),
        )
        .arg(
            Arg::new("properties")
                .short('p')
                .long("properties")
                .required(false)
                .default_value("{}")
                .help("Message properties"),
        )]
}

pub fn get_subcommands() -> [Command; 1] {
    [Command::new("messages")
        .about(color_print::cstr!("Fetches (via <red>polling, very inefficiently</red>) message(s) from a queue. <bold><red>Only suitable for development and test environments</red></bold>"))
        .after_long_help(color_print::cformat!("<bold>Doc guide</bold>: {}", POLLING_CONSUMER_GUIDE_URL))
        .arg(
            Arg::new("queue")
                .short('q')
                .long("queue")
                .required(true)
                .help("Target queue or stream name"),
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .required(false)
                .default_value("1")
                .help("Maximum number of messages to consume"),
        )
        .arg(
            Arg::new("ack_mode")
                .short('a')
                .long("ack-mode")
                .required(false)
                .default_value("ack_requeue_false")
                .help("Accepted values are: ack_requeue_false, reject_requeue_false, ack_requeue_true, reject_requeue_true"),
        )]
}

pub fn shovel_subcommands() -> [Command; 4] {
    let list_all_cmd = Command::new("list_all")
        .long_about("Lists shovels in all virtual hosts")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            SHOVEL_GUIDE_URL
        ));

    let declare_091_cmd = Command::new("declare_amqp091")
        .long_about(
            "Declares a dynamic shovel that uses AMQP 0-9-1 for both source and destination",
        )
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            SHOVEL_GUIDE_URL
        ))
        .arg(Arg::new("name").short('n').long("name").required(true))
        .arg(Arg::new("source_uri").long("source-uri").required(true))
        .arg(
            Arg::new("destination_uri")
                .long("destination-uri")
                .required(true),
        )
        .arg(
            Arg::new("ack_mode")
                .long("ack-mode")
                .help("One of: on-confirm, on-publish, no-ack")
                .default_value("on-confirm")
                .value_parser(value_parser!(ShovelAcknowledgementMode)),
        )
        .arg(
            Arg::new("source_queue")
                .long("source-queue")
                .conflicts_with("source_exchange"),
        )
        .arg(
            Arg::new("source_exchange")
                .long("source-exchange")
                .conflicts_with("source_queue"),
        )
        .arg(
            Arg::new("source_exchange_key")
                .long("source-exchange-routing-key")
                .conflicts_with("source_queue")
                .requires("source_exchange"),
        )
        .group(
            ArgGroup::new("source")
                .args(["source_queue", "source_exchange"])
                .required(true),
        )
        .arg(
            Arg::new("destination_queue")
                .long("destination-queue")
                .conflicts_with("destination_exchange"),
        )
        .arg(
            Arg::new("destination_exchange")
                .long("destination-exchange")
                .conflicts_with("destination_queue"),
        )
        .arg(
            Arg::new("destination_exchange_key")
                .long("destination-exchange-routing-key")
                .conflicts_with("destination_queue"),
        )
        .arg(
            Arg::new("predeclared_source")
                .long("predeclared-source")
                .help("The source topology will be pre-declared (should not be declared by the shovel)")
                .value_parser(value_parser!(bool))
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("predeclared_destination")
                .long("predeclared-destination")
                .help("The destination topology will be pre-declared (should not be declared by the shovel)")
                .value_parser(value_parser!(bool))
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("reconnect_delay")
                .long("reconnect-delay")
                .default_value("5")
                .value_parser(value_parser!(u16)),
        )
        .group(
            ArgGroup::new("destination")
                .args(["destination_queue", "destination_exchange"])
                .required(true),
        )
        .arg(
            Arg::new("publish_properties")
                .long("publish-properties")
                .help("A JSON object with message properties for the Shovel to set")
                .required(false)
                .default_value("{}")
                .value_parser(value_parser!(String)),
        );

    let declare_10_cmd = Command::new("declare_amqp10")
        .long_about("Declares a dynamic shovel that uses AMQP 1.0 for both source and destination")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            SHOVEL_GUIDE_URL
        ))
        .arg(Arg::new("name").short('n').long("name").required(true))
        .arg(Arg::new("source_uri").long("source-uri").required(true))
        .arg(
            Arg::new("destination_uri")
                .long("destination-uri")
                .required(true),
        )
        .arg(
            Arg::new("ack_mode")
                .long("ack-mode")
                .help("One of: on-confirm, on-publish, no-ack")
                .default_value("on-confirm")
                .value_parser(value_parser!(ShovelAcknowledgementMode)),
        )
        .arg(Arg::new("source_address").long("source-address"))
        .arg(Arg::new("destination_address").long("destination-address"))
        .arg(
            Arg::new("reconnect_delay")
                .long("reconnect-delay")
                .default_value("5")
                .value_parser(value_parser!(u16)),
        );

    let delete_cmd = Command::new("delete")
        .long_about("Deletes a dynamic shovel")
        .after_long_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            SHOVEL_GUIDE_URL
        ))
        .arg(
            Arg::new("name")
                .long("name")
                .help("shovel name (identifier)")
                .required(true),
        );

    [list_all_cmd, declare_091_cmd, declare_10_cmd, delete_cmd]
}
