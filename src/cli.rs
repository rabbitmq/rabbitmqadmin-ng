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
use crate::config::PreFlightSettings;
use crate::output::TableStyle;
use clap::{Arg, ArgAction, ArgGroup, Command, crate_name, crate_version, value_parser};
use rabbitmq_http_client::commons::{
    BindingDestinationType, ChannelUseMode, ExchangeType, MessageTransferAcknowledgementMode,
    PolicyTarget, QueueType, SupportedProtocol,
};
use rabbitmq_http_client::password_hashing::HashingAlgorithm;
use rabbitmq_http_client::requests::FederationResourceCleanupMode;

pub fn parser(pre_flight_settings: PreFlightSettings) -> Command {
    let after_help = color_print::cformat!(
        r#"
<bold><blue>Documentation and Community Resources</blue></bold>

  rabbitmqadmin docs: {}
  RabbitMQ docs: {}
  GitHub Discussions: {}
  Discord server: {}

<bold><blue>Contribute</blue></bold>

  On GitHub: {}"#,
        RABBITMQADMIN_DOC_GUIDE_URL,
        RABBITMQ_DOC_GUIDES_URL,
        GITHUB_DISCUSSIONS_URL,
        DISCORD_SERVER_INVITATION_URL,
        GITHUB_REPOSITORY_URL
    );

    let bindings_group = Command::new("bindings")
        .about("Operations on bindings")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .subcommand_value_name("binding")
        .arg_required_else_help(true)
        .subcommands(binding_subcommands(pre_flight_settings.clone()));
    let channels_group = Command::new("channels")
        .about("Operations on channels")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .arg_required_else_help(true)
        .subcommands(channels_subcommands(pre_flight_settings.clone()));
    let close_group = Command::new("close")
        .about("Closes connections")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .arg_required_else_help(true)
        .subcommands(close_subcommands(pre_flight_settings.clone()));
    let connections_group = Command::new("connections")
        .about("Operations on connections")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .arg_required_else_help(true)
        .subcommands(connections_subcommands(pre_flight_settings.clone()));
    let declare_group = Command::new("declare")
        .about("Creates or declares objects")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .arg_required_else_help(true)
        .subcommands(declare_subcommands(pre_flight_settings.clone()));
    let definitions_group = Command::new("definitions")
        .about("Operations on definitions (everything except for messages: virtual hosts, queues, streams, exchanges, bindings, users, etc)")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
                    "<bold>Doc guide</bold>: {}",
                    DEFINITION_GUIDE_URL
                ))
        .subcommand_value_name("export")
        .arg_required_else_help(true)
        .subcommands(definitions_subcommands(pre_flight_settings.clone()));
    let delete_group = Command::new("delete")
        .about("Deletes objects")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .arg_required_else_help(true)
        .subcommands(delete_subcommands(pre_flight_settings.clone()));
    let deprecated_features_group = Command::new("deprecated_features")
        .about("Operations on deprecated features")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEPRECATED_FEATURE_GUIDE_URL
        ))
        .subcommand_value_name("deprecated feature")
        .arg_required_else_help(true)
        .subcommands(deprecated_features_subcommands(pre_flight_settings.clone()));
    let exchanges_group = Command::new("exchanges")
        .about("Operations on exchanges")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .subcommand_value_name("exchange")
        .arg_required_else_help(true)
        .subcommands(exchanges_subcommands(pre_flight_settings.clone()));
    let export_group = Command::new("export")
        .about("See 'definitions export'")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEFINITION_GUIDE_URL
        ))
        .subcommand_value_name("definitions")
        .arg_required_else_help(true)
        .subcommands(export_subcommands(pre_flight_settings.clone()));
    let feature_flags_group = Command::new("feature_flags")
        .about("Operations on feature flags")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            FEATURE_FLAG_GUIDE_URL
        ))
        .subcommand_value_name("feature flag")
        .arg_required_else_help(true)
        .subcommands(feature_flags_subcommands(pre_flight_settings.clone()));
    let federation_group = Command::new("federation")
        .about("Operations on federation upstreams and links")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            r#"<bold>Doc guides</bold>:

 * {}
 * {}
 * {}
 * {}"#,
            FEDERATION_GUIDE_URL,
            FEDERATED_EXCHANGES_GUIDE_URL,
            FEDERATED_QUEUES_GUIDE_URL,
            FEDERATION_REFERENCE_URL
        ))
        .arg_required_else_help(true)
        .subcommands(federation_subcommands(pre_flight_settings.clone()));
    let get_group = Command::new("get")
        .about(color_print::cstr!("Fetches message(s) from a queue or stream via <bold><red>polling</red></bold>. <bold><red>Only suitable for development and test environments</red></bold>."))
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!("<bold>Doc guide</bold>: {}", POLLING_CONSUMER_GUIDE_URL))
        .subcommand_value_name("message")
        .arg_required_else_help(true)
        .subcommands(get_subcommands(pre_flight_settings.clone()));
    let global_parameters_group = Command::new("global_parameters")
        .about("Operations on global runtime parameters")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            RUNTIME_PARAMETER_GUIDE_URL
        ))
        .subcommand_value_name("runtime_parameter")
        .arg_required_else_help(true)
        .subcommands(global_parameters_subcommands(pre_flight_settings.clone()));
    let health_check_group = Command::new("health_check")
        .about("Runs health checks")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .subcommand_value_name("check")
        .arg_required_else_help(true)
        .subcommands(health_check_subcommands(pre_flight_settings.clone()))
        .after_help(color_print::cformat!(
            r#"<bold>Doc guides</bold>:

 * {}
 * {}"#,
            HEALTH_CHECK_GUIDE_URL,
            DEPRECATED_FEATURE_GUIDE_URL
        ));
    let import_group = Command::new("import")
        .about("See 'definitions import'")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEFINITION_GUIDE_URL
        ))
        .subcommand_value_name("definitions")
        .arg_required_else_help(true)
        .subcommands(import_subcommands(pre_flight_settings.clone()));
    let list_group = Command::new("list")
        .about("Lists objects")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .arg_required_else_help(true)
        .subcommands(list_subcommands(pre_flight_settings.clone()));
    let nodes_group = Command::new("nodes")
        .about("Node operations")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .arg_required_else_help(true)
        .subcommands(nodes_subcommands(pre_flight_settings.clone()));
    let operator_policies_group = Command::new("operator_policies")
        .about("Operations on operator policies")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            POLICY_GUIDE_URL
        ))
        .subcommand_value_name("operator policy")
        .arg_required_else_help(true)
        .subcommands(operator_policies_subcommands(pre_flight_settings.clone()));
    let parameters_group = Command::new("parameters")
        .about("Operations on runtime parameters")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            RUNTIME_PARAMETER_GUIDE_URL
        ))
        .subcommand_value_name("runtime_parameter")
        .arg_required_else_help(true)
        .subcommands(parameters_subcommands(pre_flight_settings.clone()));
    let passwords_group = Command::new("passwords")
        .about("Operations on passwords")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            PASSWORD_GUIDE_URL
        ))
        .arg_required_else_help(true)
        .subcommands(passwords_subcommands(pre_flight_settings.clone()));
    let permissions_group = Command::new("permissions")
        .about("Operations on user permissions")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            ACCESS_CONTROL_GUIDE_URL
        ))
        .subcommand_value_name("permission")
        .arg_required_else_help(true)
        .subcommands(permissions_subcommands(pre_flight_settings.clone()));
    let plugins_group = Command::new("plugins")
        .about("List enabled plugins")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            PLUGIN_GUIDE_URL
        ))
        .subcommand_value_name("plugin")
        .arg_required_else_help(true)
        .subcommands(plugins_subcommands(pre_flight_settings.clone()));
    let policies_group = Command::new("policies")
        .about("Operations on policies")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            POLICY_GUIDE_URL
        ))
        .subcommand_value_name("policy")
        .arg_required_else_help(true)
        .subcommands(policies_subcommands(pre_flight_settings.clone()));
    let publish_group = Command::new("publish")
        .about(color_print::cstr!("Publishes (<red>inefficiently</red>) message(s) to a queue or a stream. <bold><red>Only suitable for development and test environments</red></bold>."))
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!("<bold>Doc guide</bold>: {}", PUBLISHER_GUIDE_URL))
        .subcommand_value_name("message")
        .arg_required_else_help(true)
        .subcommands(publish_subcommands(pre_flight_settings.clone()));
    let purge_group = Command::new("purge")
        .about("Purges queues")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .subcommand_value_name("queue")
        .arg_required_else_help(true)
        .subcommands(purge_subcommands(pre_flight_settings.clone()));
    let queues_group = Command::new("queues")
        .about("Operations on queues")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .subcommand_value_name("queue")
        .arg_required_else_help(true)
        .subcommands(queues_subcommands(pre_flight_settings.clone()));
    let rebalance_group = Command::new("rebalance")
        .about("Rebalancing of leader replicas")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            QUORUM_QUEUE_GUIDE_URL
        ))
        .subcommand_value_name("queues")
        .arg_required_else_help(true)
        .subcommands(rebalance_subcommands(pre_flight_settings.clone()));
    let show_group = Command::new("show")
        .about("Overview, memory footprint breakdown, and more")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            MONITORING_GUIDE_URL
        ))
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .arg_required_else_help(true)
        .subcommands(show_subcommands(pre_flight_settings.clone()));
    let shovels_group = Command::new("shovels")
        .about("Operations on shovels")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            SHOVEL_GUIDE_URL
        ))
        .subcommand_value_name("shovels")
        .arg_required_else_help(true)
        .subcommands(shovel_subcommands(pre_flight_settings.clone()));
    let streams_group = Command::new("streams")
        .about("Operations on streams")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .subcommand_value_name("stream")
        .arg_required_else_help(true)
        .subcommands(streams_subcommands(pre_flight_settings.clone()));
    let tanzu_group = Command::new("tanzu")
        .about("Tanzu RabbitMQ-specific commands")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            COMMERCIAL_OFFERINGS_GUIDE_URL
        ))
        .subcommand_value_name("subcommand")
        .arg_required_else_help(true)
        .subcommands(tanzu_subcommands());
    let users_group = Command::new("users")
        .about("Operations on users")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            ACCESS_CONTROL_GUIDE_URL
        ))
        .subcommand_value_name("subcommand")
        .arg_required_else_help(true)
        .subcommands(users_subcommands(pre_flight_settings.clone()));
    let user_limits_group = Command::new("user_limits")
        .about("Operations on per-user (resource) limits")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            USER_LIMIT_GUIDE_URL
        ))
        .subcommand_value_name("user_limit")
        .arg_required_else_help(true)
        .subcommands(user_limits_subcommands(pre_flight_settings.clone()));
    let vhosts_group = Command::new("vhosts")
        .about("Virtual host operations")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .arg_required_else_help(true)
        .subcommands(vhosts_subcommands(pre_flight_settings.clone()));
    let vhost_limits_group = Command::new("vhost_limits")
        .about("Operations on virtual host (resource) limits")
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            VIRTUAL_HOST_LIMIT_GUIDE_URL
        ))
        .subcommand_value_name("vhost_limit")
        .arg_required_else_help(true)
        .subcommands(vhost_limits_subcommands(pre_flight_settings.clone()));

    let command_groups = [
        bindings_group,
        channels_group,
        close_group,
        connections_group,
        declare_group,
        definitions_group,
        delete_group,
        deprecated_features_group,
        exchanges_group,
        export_group,
        feature_flags_group,
        federation_group,
        get_group,
        global_parameters_group,
        health_check_group,
        import_group,
        list_group,
        nodes_group,
        operator_policies_group,
        parameters_group,
        passwords_group,
        permissions_group,
        plugins_group,
        policies_group,
        publish_group,
        purge_group,
        queues_group,
        rebalance_group,
        show_group,
        shovels_group,
        streams_group,
        tanzu_group,
        users_group,
        user_limits_group,
        vhosts_group,
        vhost_limits_group,
    ];

    Command::new(crate_name!())
        .version(crate_version!())
        .author("The RabbitMQ Core Team")
        .about(format!("{} gen 2, version: {}", crate_name!(), crate_version!()))
        .long_about(format!(
            "RabbitMQ CLI that uses the HTTP API. Version: {}",
            crate_version!()
        ))
        .infer_subcommands(pre_flight_settings.infer_subcommands)
        .infer_long_args(pre_flight_settings.infer_long_options)
        .after_help(after_help)
        .disable_version_flag(true)
        // --config-file
        .arg(
            Arg::new("config_file_path")
                .short('c')
                .long("config")
                .env("RABBITMQADMIN_CONFIG_FILE_PATH")
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
                .env("RABBITMQADMIN_NODE_ALIAS")
                .required(false)
                .default_value(DEFAULT_NODE_ALIAS),
        )
        // --host
        .arg(
            Arg::new("host")
                .short('H')
                .long("host")
                .alias("hostname")
                .env("RABBITMQADMIN_TARGET_HOST")
                .help("HTTP API hostname to use when connecting"),
        )
        .visible_alias("hostname")
        // --port
        .arg(
            Arg::new("port")
                .short('P')
                .long("port")
                .env("RABBITMQADMIN_TARGET_PORT")
                .help("HTTP API port to use when connecting")
                .required(false)
                .value_parser(value_parser!(u16)),
        )
        // --base-uri
        .arg(
            Arg::new("base_uri")
                .short('U')
                .long("base-uri")
                .env("RABBITMQADMIN_BASE_URI")
                .help("base HTTP API endpoint URI")
                .required(false)
                .conflicts_with_all(["host", "port"]),
        )
        // --path-prefix
        .arg(
            Arg::new("path_prefix")
                .long("path-prefix")
                .env("RABBITMQADMIN_API_PATH_PREFIX")
                .help("use if target node uses a path prefix. Defaults to '/api'"),
        )
        // --vhost
        .arg(
            Arg::new("vhost")
                .short('V')
                .long("vhost")
                // IMPORTANT: this means that subcommands won't be able to override --vhost or -V,
                // otherwise the parser will panic. MK.
                .global(true)
                .env("RABBITMQADMIN_TARGET_VHOST")
                .help("target virtual host. Defaults to '/'"),
        )
        // --username
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .env("RABBITMQADMIN_USERNAME")
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
                .env("RABBITMQADMIN_PASSWORD")
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
                .env("RABBITMQADMIN_USE_TLS")
                .value_parser(value_parser!(bool))
                .action(ArgAction::SetTrue),
        )
        // --tls-ca-cert-file
        .arg(
            Arg::new("ca_certificate_bundle_path")
                .long("tls-ca-cert-file")
                .required(false)
                .help("Local path to a CA certificate file in the PEM format")
                .value_parser(value_parser!(PathBuf)),
        )
        // --tls-cert-file
        .arg(
            Arg::new("client_certificate_file_path")
                .long("tls-cert-file")
                .required(false)
                .requires("tls")
                .help("Local path to a client certificate file in the PEM format")
                .value_parser(value_parser!(PathBuf)),
        )
        // --tls-key-file
        .arg(
            Arg::new("client_private_key_file_path")
                .long("tls-key-file")
                .required(false)
                .requires("tls")
                .help("Local path to a client private key file in the PEM format")
                .value_parser(value_parser!(PathBuf)),
        )
        // --timeout
        .arg(
            Arg::new("timeout")
                .long("timeout")
                .env("RABBITMQADMIN_TIMEOUT")
                .help("HTTP API request timeout in seconds. Must be greater than 0")
                .required(false)
                .default_value("60")
                .value_parser(value_parser!(u64).range(1..)),
        )
        // --quiet
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .env("RABBITMQADMIN_QUIET_MODE")
                .help("produce less output")
                .required(false)
                .value_parser(value_parser!(bool))
                .action(ArgAction::SetTrue),
        )
        // --non-interactive
        .arg(
            Arg::new("non_interactive")
                .long("non-interactive")
                .global(true)
                .env("RABBITMQADMIN_NON_INTERACTIVE_MODE")
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
                .global(true)
                .env("RABBITMQADMIN_TABLE_STYLE")
                .help("style preset to apply to output tables: modern, borderless, ascii, dots, psql, markdown, sharp")
                .conflicts_with("non_interactive")
                .required(false)
                .value_parser(value_parser!(TableStyle))
        )
        .subcommand_required(true)
        .subcommands(command_groups)
}

fn list_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let nodes_cmd = Command::new("nodes").long_about("Lists cluster members");
    let vhosts_cmd = Command::new("vhosts")
        .long_about("Lists virtual hosts")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            VIRTUAL_HOST_GUIDE_URL
        ));
    let vhost_limits_cmd = Command::new("vhost_limits")
        .long_about("Lists virtual host (resource) limits")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            VIRTUAL_HOST_GUIDE_URL
        ));
    let connections_cmd = Command::new("connections")
        .long_about("Lists client connections")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            CONNECTION_GUIDE_URL
        ));
    let channels_cmd = Command::new("channels")
        .long_about("Lists AMQP 0-9-1 channels")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            CHANNEL_GUIDE_URL
        ));
    let queues_cmd = Command::new("queues")
        .long_about("Lists queues and streams")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            QUEUE_GUIDE_URL
        ));
    let exchanges_cmd = Command::new("exchanges").long_about("Lists exchanges");
    let bindings_cmd = Command::new("bindings").long_about("Lists bindings");
    let consumers_cmd = Command::new("consumers")
        .long_about("Lists consumers")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            CONSUMER_GUIDE_URL
        ));
    let parameters_cmd = Command::new("parameters")
        .arg(
            Arg::new("component")
                .long("component")
                .help("component (for example: federation-upstream, vhost-limits)")
                .required(false),
        )
        .long_about("Lists runtime parameters")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            RUNTIME_PARAMETER_GUIDE_URL
        ));
    let policies_cmd = Command::new("policies")
        .long_about("Lists policies")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            POLICY_GUIDE_URL
        ));
    let operator_policies_cmd = Command::new("operator_policies")
        .long_about("Lists operator policies")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            OPERATOR_POLICY_GUIDE_URL
        ));
    let users_cmd = Command::new("users").long_about("Lists users in the internal database");
    let permissions_cmd = Command::new("permissions")
        .long_about("Lists user permissions")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            ACCESS_CONTROL_GUIDE_URL
        ));
    let user_connections_cmd = Command::new("user_connections")
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .required(true)
                .help("Name of the user whose connections should be listed"),
        )
        .long_about("Lists client connections that authenticated with a specific username")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            CONNECTION_GUIDE_URL
        ));
    let user_limits_cmd = Command::new("user_limits")
        .arg(
            Arg::new("user")
                .long("user")
                .help("username")
                .required(false),
        )
        .long_about("Lists per-user (resource) limits")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            USER_LIMIT_GUIDE_URL
        ));
    let feature_flags_cmd = Command::new("feature_flags")
        .long_about("Lists feature flags and their cluster state")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            FEATURE_FLAG_GUIDE_URL
        ));
    let deprecated_features_cmd = Command::new("deprecated_features")
        .long_about("Lists all deprecated features")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEPRECATED_FEATURE_GUIDE_URL
        ));
    let deprecated_features_in_use_cmd = Command::new("deprecated_features_in_use")
        .long_about("Lists the deprecated features that are in used in the cluster")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEPRECATED_FEATURE_GUIDE_URL
        ));
    [
        nodes_cmd,
        users_cmd,
        vhosts_cmd,
        permissions_cmd,
        connections_cmd,
        user_connections_cmd,
        channels_cmd,
        queues_cmd,
        exchanges_cmd,
        bindings_cmd,
        consumers_cmd,
        parameters_cmd,
        policies_cmd,
        operator_policies_cmd,
        vhost_limits_cmd,
        user_limits_cmd,
        feature_flags_cmd,
        deprecated_features_cmd,
        deprecated_features_in_use_cmd,
    ]
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

fn declare_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let user_cmd = Command::new("user")
        .about("Creates a user")
        .arg(
            Arg::new("name")
                .long("name")
                .help("username")
                .required(true),
        )
        .arg(
            Arg::new("password_hash")
                .help(color_print::cformat!(
                    "salted password hash, see {}",
                    PASSWORD_GUIDE_URL
                ))
                .long("password-hash")
                .required(false)
                .default_value(""),
        )
        .arg(
            Arg::new("password")
                .long("password")
                .help(color_print::cformat!(
                    "prefer providing a hash, see {}",
                    PASSWORD_GUIDE_URL
                ))
                .required(false)
                .default_value(""),
        )
        .arg(
            Arg::new("hashing_algorithm")
                .long("hashing-algorithm")
                .required(false)
                .conflicts_with("password_hash")
                .requires("password")
                .value_parser(value_parser!(HashingAlgorithm))
                .default_value("SHA256")
                .help("The hashing algorithm to use: SHA256 or SHA512"),
        )
        .arg(
            Arg::new("tags")
                .long("tags")
                .help("a list of comma-separated tags")
                .default_value(""),
        );
    let vhost_cmd = Command::new("vhost")
        .about("Creates a virtual host")
        .after_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", VIRTUAL_HOST_GUIDE_URL))
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
        );
    let permissions_cmd = Command::new("permissions")
        .about("grants permissions to a user")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            ACCESS_CONTROL_GUIDE_URL
        ))
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
        );
    let queue_cmd = Command::new("queue")
        .about("Declares a queue or a stream")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            QUEUE_GUIDE_URL
        ))
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
        );
    let stream_cmd = Command::new("stream")
        .about("Declares a stream")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            STREAM_GUIDE_URL
        ))
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
        );
    let exchange_cmd = Command::new("exchange")
        .about("Declares an exchange")
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
        );
    let binding_cmd = Command::new("binding")
        .about("Creates a binding between a source exchange and a destination (a queue or an exchange)")
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
        );
    let parameter_cmd = Command::new("parameter")
        .about("Sets a runtime parameter")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            RUNTIME_PARAMETER_GUIDE_URL
        ))
        .arg(
            Arg::new("name")
                .long("name")
                .help("parameter's name")
                .required(true),
        )
        .arg(
            Arg::new("component")
                .long("component")
                .help("component (eg. federation)")
                .required(true),
        )
        .arg(
            Arg::new("value")
                .long("value")
                .help("parameter's value")
                .required(true),
        );
    let policy_cmd = Command::new("policy")
        .about("Creates or updates a policy")
        .after_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", POLICY_GUIDE_URL))
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
                .alias("applies-to")
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
    let operator_policy_cmd = Command::new("operator_policy")
        .about("Creates or updates an operator policy")
        .after_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", OPERATOR_POLICY_GUIDE_URL))
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
                .alias("applies-to")
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
    let vhost_limit_cmd = Command::new("vhost_limit")
        .about("Set a vhost limit")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            VIRTUAL_HOST_LIMIT_GUIDE_URL
        ))
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
        );
    let user_limit_cmd = Command::new("user_limit")
        .about("Set a user limit")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            USER_LIMIT_GUIDE_URL
        ))
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
        );
    [
        user_cmd,
        vhost_cmd,
        permissions_cmd,
        queue_cmd,
        stream_cmd,
        exchange_cmd,
        binding_cmd,
        parameter_cmd,
        policy_cmd,
        operator_policy_cmd,
        vhost_limit_cmd,
        user_limit_cmd,
    ]
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

fn show_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let overview_cmd = Command::new("overview")
        .about("Displays essential information about target node and its cluster");
    let churn_cmd = Command::new("churn").about("Displays object churn metrics");
    let endpoint_cmd = Command::new("endpoint")
        .about("Displays the computed HTTP API endpoint URI. Use for troubleshooting only.");
    let memory_breakdown_in_bytes_cmd = Command::new("memory_breakdown_in_bytes")
        .about("Provides a memory footprint breakdown (in bytes) for the target node")
        .arg(
            Arg::new("node")
                .long("node")
                .help("target node, must be a cluster member")
                .required(true),
        )
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            MEMORY_FOOTPRINT_GUIDE_URL
        ));

    let memory_breakdown_in_percent_cmd = Command::new("memory_breakdown_in_percent")
        .about("Provides a memory footprint breakdown (in percent) for the target node")
        .arg(
            Arg::new("node")
                .long("node")
                .help("target node, must be a cluster member")
                .required(true),
        )
        .after_help(color_print::cformat!(
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
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

fn delete_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);

    let user_cmd = Command::new("user")
        .about("Deletes a user")
        .arg(
            Arg::new("name")
                .long("name")
                .help("username")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    let vhost_cmd = Command::new("vhost")
        .about("Deletes a virtual host")
        .arg(
            Arg::new("name")
                .long("name")
                .help("virtual host")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    let permissions_cmd = Command::new("permissions")
        .about("Revokes user permissions to a given vhost")
        .arg(
            Arg::new("user")
                .long("user")
                .help("username")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    let queue_cmd = Command::new("queue")
        .about("Deletes a queue")
        .arg(
            Arg::new("name")
                .long("name")
                .help("queue name")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    let stream_cmd = Command::new("stream")
        .about("Deletes a stream")
        .arg(
            Arg::new("name")
                .long("name")
                .help("stream name")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    let exchange_cmd = Command::new("exchange")
        .about("Deletes an exchange")
        .arg(
            Arg::new("name")
                .long("name")
                .help("exchange name")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    let binding_cmd = Command::new("binding")
        .about("Deletes a binding")
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
        )
        .arg(idempotently_arg.clone());
    let parameter_cmd = Command::new("parameter")
        .about("Clears a runtime parameter")
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
        )
        .arg(idempotently_arg.clone());
    let policy_cmd = Command::new("policy")
        .about("Deletes a policy")
        .arg(
            Arg::new("name")
                .long("name")
                .help("policy name")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    let operator_policy_cmd = Command::new("operator_policy")
        .about("Deletes an operator policy")
        .arg(
            Arg::new("name")
                .long("name")
                .help("operator policy name")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    let vhost_limit_cmd = Command::new("vhost_limit")
        .about("delete a vhost limit")
        .arg(
            Arg::new("name")
                .long("name")
                .help("limit name (eg. max-connections, max-queues)")
                .required(true),
        );
    let user_limit_cmd = Command::new("user_limit")
        .about("Clears a user limit")
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
        );
    let shovel_cmd = Command::new("shovel")
        .about("Delete a shovel")
        .arg(idempotently_arg.clone())
        .arg(
            Arg::new("name")
                .long("name")
                .help("shovel name")
                .required(true),
        );
    [
        user_cmd,
        vhost_cmd,
        permissions_cmd,
        queue_cmd,
        stream_cmd,
        exchange_cmd,
        binding_cmd,
        parameter_cmd,
        policy_cmd,
        operator_policy_cmd,
        vhost_limit_cmd,
        user_limit_cmd,
        shovel_cmd,
    ]
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

fn purge_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let queue_cmd = Command::new("queue")
        .long_about("Purges (permanently removes unacknowledged messages from) a queue")
        .arg(
            Arg::new("name")
                .long("name")
                .help("name of the queue to purge")
                .required(true),
        );
    [queue_cmd]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

fn binding_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);

    let declare_cmd = Command::new("declare")
        .about("Creates a binding between a source exchange and a destination (a queue or an exchange)")
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
        );
    let delete_cmd = Command::new("delete")
        .about("Deletes a binding")
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
        )
        .arg(idempotently_arg.clone());
    let list_cmd = Command::new("list").long_about("Lists bindings");

    [declare_cmd, delete_cmd, list_cmd]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

fn queues_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let declare_cmd = Command::new("declare")
        .about("Declares a queue or a stream")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            QUEUE_GUIDE_URL
        ))
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
        );
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);
    let delete_cmd = Command::new("delete")
        .about("Deletes a queue")
        .arg(
            Arg::new("name")
                .long("name")
                .help("queue name")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    let list_cmd = Command::new("list")
        .long_about("Lists queues and streams")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            QUEUE_GUIDE_URL
        ));
    let purge_cmd = Command::new("purge")
        .long_about("Purges (permanently removes unacknowledged messages from) a queue")
        .arg(
            Arg::new("name")
                .long("name")
                .help("name of the queue to purge")
                .required(true),
        );
    let rebalance_cmd = Command::new("rebalance").about("Rebalances queue leaders");
    [declare_cmd, delete_cmd, list_cmd, purge_cmd, rebalance_cmd]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

fn streams_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let declare_cmd = Command::new("declare")
        .about("Declares a stream")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            STREAM_GUIDE_URL
        ))
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
        );
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);
    let delete_cmd = Command::new("delete")
        .about("Deletes a queue")
        .arg(
            Arg::new("name")
                .long("name")
                .help("queue name")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    let list_cmd = Command::new("list")
        .long_about("Lists streams and queues and")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            STREAM_GUIDE_URL
        ));
    [declare_cmd, delete_cmd, list_cmd]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

fn parameters_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);

    let list_all_cmd = Command::new("list_all")
        .long_about("Lists all runtime parameters across all virtual hosts")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            RUNTIME_PARAMETER_GUIDE_URL
        ));
    let list_cmd = Command::new("list")
        .arg(
            Arg::new("component")
                .long("component")
                .help("component (for example: federation-upstream, vhost-limits)")
                .required(false),
        )
        .long_about("Lists runtime parameters")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            RUNTIME_PARAMETER_GUIDE_URL
        ));
    let list_in_cmd = Command::new("list_in")
        .arg(
            Arg::new("component")
                .long("component")
                .help("component (for example: federation-upstream, vhost-limits)")
                .required(true),
        )
        .long_about("Lists runtime parameters")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            RUNTIME_PARAMETER_GUIDE_URL
        ));

    let set_cmd = Command::new("set")
        .alias("declare")
        .about("Sets a runtime parameter")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            RUNTIME_PARAMETER_GUIDE_URL
        ))
        .arg(
            Arg::new("name")
                .long("name")
                .help("parameter's name")
                .required(true),
        )
        .arg(
            Arg::new("component")
                .long("component")
                .help("component (eg. federation)")
                .required(true),
        )
        .arg(
            Arg::new("value")
                .long("value")
                .help("parameter's value")
                .required(true),
        );

    let clear_cmd = Command::new("clear")
        .alias("delete")
        .about("Clears (deletes) a runtime parameter")
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
        )
        .arg(idempotently_arg.clone());

    [clear_cmd, list_all_cmd, list_cmd, list_in_cmd, set_cmd]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

fn global_parameters_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);

    let list_cmd = Command::new("list")
        .long_about("Lists global runtime parameters")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            RUNTIME_PARAMETER_GUIDE_URL
        ));

    let set_cmd = Command::new("set")
        .alias("declare")
        .about("Sets a global runtime parameter")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            RUNTIME_PARAMETER_GUIDE_URL
        ))
        .arg(
            Arg::new("name")
                .long("name")
                .help("parameter's name")
                .required(true),
        )
        .arg(
            Arg::new("value")
                .long("value")
                .help("parameter's value")
                .required(true),
        );

    let clear_cmd = Command::new("clear")
        .alias("delete")
        .about("Clears (deletes) a global runtime parameter")
        .arg(
            Arg::new("name")
                .long("name")
                .help("parameter's name")
                .required(true),
        )
        .arg(idempotently_arg.clone());

    [clear_cmd, list_cmd, set_cmd]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

fn operator_policies_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);

    let declare_cmd = Command::new("declare")
        .visible_aliases(vec!["update", "set"])
        .about("Creates or updates an operator policy")
        .after_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", POLICY_GUIDE_URL))
        .arg(
            Arg::new("name")
                .long("name")
                .help("operator policy name")
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
                .alias("applies-to")
                .help("entities to apply to (queues, classic_queues, quorum_queues, streams, exchanges, all)")
                .value_parser(value_parser!(PolicyTarget))
                .required(true),
        )
        .arg(
            Arg::new("priority")
                .long("priority")
                .help("operator policy priority (only the policy with the highest priority is effective)")
                .required(false)
                .default_value("0"),
        )
        .arg(
            Arg::new("definition")
                .long("definition")
                .help("operator policy definition")
                .required(true),
        );

    let list_cmd = Command::new("list")
        .long_about("Lists operator policies")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            POLICY_GUIDE_URL
        ));

    let delete_cmd = Command::new("delete")
        .about("Deletes an operator policy")
        .arg(
            Arg::new("name")
                .long("name")
                .help("policy name")
                .required(true),
        )
        .arg(idempotently_arg.clone());

    let delete_definition_key_cmd = Command::new("delete_definition_keys")
        .about("Deletes definition keys from an operator policy, unless it is the only key")
        .arg(
            Arg::new("name")
                .long("name")
                .help("operator policy name")
                .required(true),
        )
        .arg(
            Arg::new("definition_keys")
                .long("definition-keys")
                .num_args(1..)
                .value_delimiter(',')
                .action(ArgAction::Append)
                .help("comma-separated definition keys"),
        );

    let delete_definition_key_from_all_in_cmd = Command::new("delete_definition_keys_from_all_in")
        .about("Deletes a definition key from all operator policies in a virtual host, unless it is the only key")
        .arg(
            Arg::new("definition_keys")
                .long("definition-keys")
                .num_args(1..)
                .value_delimiter(',')
                .action(ArgAction::Append)
                .help("comma-separated definition keys")
        );

    let list_in_cmd = Command::new("list_in")
        .about("Lists operator policies in a specific virtual host")
        .arg(
            Arg::new("apply_to")
                .long("apply-to")
                .alias("applies-to")
                .value_parser(value_parser!(PolicyTarget)),
        );

    let list_matching_cmd = Command::new("list_matching_object")
        .about("Lists operator policies that match an object (queue, stream, exchange) name")
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

    let patch_cmd = Command::new("patch")
        .about("Merges a set of keys into existing operator policy definitions")
        .arg(
            Arg::new("name")
                .long("name")
                .help("operator policy name")
                .required(true),
        )
        .arg(
            Arg::new("definition")
                .long("definition")
                .help("operator policy definition changes to merge into the existing ones"),
        );

    let update_cmd = Command::new("update_definition")
        .about("Updates an operator policy definition key")
        .arg(
            Arg::new("name")
                .long("name")
                .help("operator policy name")
                .required(true),
        )
        .arg(
            Arg::new("definition_key")
                .long("definition-key")
                .help("operator policy definition key to update")
                .required(true),
        )
        .arg(
            Arg::new("definition_value")
                .long("new-value")
                .help("new definition value to set")
                .required(true),
        );

    let update_all_in_cmd = Command::new("update_definitions_of_all_in")
        .about("Updates a definition key in all operator policies in a virtual host")
        .arg(
            Arg::new("definition_key")
                .long("definition-key")
                .help("operator policy definition key to update")
                .required(true),
        )
        .arg(
            Arg::new("definition_value")
                .long("new-value")
                .help("new operator definition value to set")
                .required(true),
        );

    [
        declare_cmd,
        delete_cmd,
        delete_definition_key_cmd,
        delete_definition_key_from_all_in_cmd,
        list_cmd,
        list_in_cmd,
        list_matching_cmd,
        patch_cmd,
        update_cmd,
        update_all_in_cmd,
    ]
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

fn policies_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);

    let declare_cmd = Command::new("declare")
        .visible_aliases(vec!["update", "set"])
        .about("Creates or updates a policy")
        .after_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", POLICY_GUIDE_URL))
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
                .alias("applies-to")
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

    let declare_override_cmd = Command::new("declare_override")
        .about("Declares a new policy from an existing one, with a higher priority, and merges a set of keys into the new overriding policy definition")
        .arg(
            Arg::new("name")
                .long("name")
                .help("the name of the policy to create an override for")
                .required(true),
        )
        .arg(
            Arg::new("override_name")
                .long("override-name")
                .help("the name of the new overriding policy. If omitted, an 'override' suffix will be added to the original name.")
                .required(false),
        )
        .arg(
            Arg::new("definition")
                .long("definition")
                .help("additional definitions to merge into the new overriding policy"),
        );

    let declare_blanket_cmd = Command::new("declare_blanket")
        .about("Creates a low priority blanket policy, a policy that matches all objects not matched by any other policy")
        .after_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", POLICY_GUIDE_URL))
        .arg(
            Arg::new("name")
                .long("name")
                .help("blanket policy name")
                .required(true),
        )
        .arg(
            Arg::new("apply_to")
                .long("apply-to")
                .alias("applies-to")
                .help("entities to apply to (queues, classic_queues, quorum_queues, streams, exchanges, all)")
                .value_parser(value_parser!(PolicyTarget))
                .required(true),
        )
        .arg(
            Arg::new("definition")
                .long("definition")
                .help("policy definition")
                .required(true),
        );

    let list_cmd = Command::new("list")
        .long_about("Lists policies")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            POLICY_GUIDE_URL
        ));

    let delete_cmd = Command::new("delete")
        .about("Deletes a policy")
        .arg(
            Arg::new("name")
                .long("name")
                .help("policy name")
                .required(true),
        )
        .arg(idempotently_arg.clone());

    let delete_definition_keys_cmd = Command::new("delete_definition_keys")
        .about("Deletes a definition key from a policy, unless it is the only key")
        .arg(
            Arg::new("name")
                .long("name")
                .help("policy name")
                .required(true),
        )
        .arg(
            Arg::new("definition_keys")
                .long("definition-keys")
                .num_args(1..)
                .value_delimiter(',')
                .action(ArgAction::Append)
                .help("comma-separated definition keys"),
        );

    let delete_definition_keys_from_all_in_cmd = Command::new("delete_definition_keys_from_all_in")
        .about("Deletes definition keys from all policies in a virtual host, unless it is the only policy key")
        .arg(
            Arg::new("definition_keys")
                .long("definition-keys")
                .help("comma-separated definition keys")
                .num_args(1..)
                .value_delimiter(',')
                .action(ArgAction::Append)
                .required(true)
        );

    let list_in_cmd = Command::new("list_in")
        .about("Lists policies in a specific virtual host")
        .arg(
            Arg::new("apply_to")
                .long("apply-to")
                .alias("applies-to")
                .value_parser(value_parser!(PolicyTarget)),
        );

    let list_matching_cmd = Command::new("list_matching_object")
        .about("Lists policies that match an object (queue, stream, exchange) name")
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

    let patch_cmd = Command::new("patch")
        .about("Merges a set of keys into existing policy definitions")
        .arg(
            Arg::new("name")
                .long("name")
                .help("policy name")
                .required(true),
        )
        .arg(
            Arg::new("definition")
                .long("definition")
                .help("policy definition changes to merge into the existing ones"),
        );

    let update_cmd = Command::new("update_definition")
        .about("Updates a policy definition key")
        .arg(
            Arg::new("name")
                .long("name")
                .help("policy name")
                .required(true),
        )
        .arg(
            Arg::new("definition_key")
                .long("definition-key")
                .help("policy definition key to update")
                .required(true),
        )
        .arg(
            Arg::new("definition_value")
                .long("new-value")
                .help("new definition value to set")
                .required(true),
        );

    let update_all_in_cmd = Command::new("update_definitions_of_all_in")
        .about("Updates a definition key in all policies in a virtual host")
        .arg(
            Arg::new("definition_key")
                .long("definition-key")
                .help("policy definition key to update")
                .required(true),
        )
        .arg(
            Arg::new("definition_value")
                .long("new-value")
                .help("new definition value to set")
                .required(true),
        );

    [
        declare_cmd,
        declare_override_cmd,
        declare_blanket_cmd,
        delete_cmd,
        delete_definition_keys_cmd,
        delete_definition_keys_from_all_in_cmd,
        list_cmd,
        list_in_cmd,
        list_matching_cmd,
        patch_cmd,
        update_cmd,
        update_all_in_cmd,
    ]
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

fn health_check_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let node_is_quorum_critical_after_help = color_print::cformat!(
        r#"
<bold>Doc guides</bold>:

 * {}
 * {}"#,
        QUORUM_QUEUE_FAILURE_HANDLING_GUIDE_URL,
        UPGRADE_GUIDE_URL
    );

    let local_alarms = Command::new("local_alarms")
        .about("Checks if there are any resource alarms in effect on the target node");
    let cluster_wide_alarms = Command::new("cluster_wide_alarms")
        .about("Checks if there are any resource alarms in effect across the entire cluster");
    let node_is_quorum_critical = Command::new("node_is_quorum_critical")
        .about("Fails if there are queues/streams with minimum online quorum (queues/streams that will lose their quorum if the target node shuts down)")
        .after_help(node_is_quorum_critical_after_help);
    let deprecated_features_in_use = Command::new("deprecated_features_in_use")
        .about("Fails if there are any deprecated features in use in the cluster")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEPRECATED_FEATURE_GUIDE_URL
        ));

    let port_listener = Command::new("port_listener")
        .about(
            "Verifies that there's a reachable TCP listener on the given port on the target node",
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_parser(value_parser!(u16)),
        )
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            HEALTH_CHECK_GUIDE_URL
        ));

    let protocol_listener = Command::new("protocol_listener")
        .about(
            "Verifies that there's a reachable TCP listener on the given protocol alias on the target node",
        )
        .arg(
            Arg::new("protocol")
                .long("protocol")
                .value_parser(value_parser!(SupportedProtocol))
                .long_help("An alias for one of the protocols that RabbitMQ supports, with or without TLS: 'amqp', 'amqp/ssl', 'stream', 'stream/ssl', 'mqtt', 'mqtt/ssl', 'stomp', 'stomp/ssl', 'http/web-mqtt', 'https/web-mqtt', 'http/web-stomp', 'https/web-stomp', 'http/prometheus', 'https/prometheus', 'http', 'https'"),
        )
        .after_help(color_print::cformat!(
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
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

fn rebalance_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let queues_cmd = Command::new("queues").about("Rebalances queue leaders");
    [queues_cmd]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

fn close_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);

    let close_connection = Command::new("connection")
        .about("Closes a client connection")
        .arg(
            Arg::new("name")
                .long("name")
                .help("connection name (identifying string)")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    let close_user_connections = Command::new("user_connections")
        .about("Closes all connections that authenticated with a specific username")
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .help("Name of the user whose connections to close")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    [close_connection, close_user_connections]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

fn channels_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let list_cmd = Command::new("list")
        .long_about("Lists all channels across all virtual hosts")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            "https://www.rabbitmq.com/docs/channels"
        ));

    [list_cmd]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

fn connections_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);

    let close_connection = Command::new("close")
        .about("Closes a client connection")
        .arg(
            Arg::new("name")
                .long("name")
                .help("connection name (identifying string)")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    let close_user_connections = Command::new("close_of_user")
        .about("Closes all connections that are authenticated with a specific username")
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .help("Name of the user whose connections should be closed")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    let list_cmd = Command::new("list")
        .long_about("Lists client connections")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            CONNECTION_GUIDE_URL
        ));
    let list_user_connections_cmd = Command::new("list_of_user")
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .required(true)
                .help("Name of the user whose connections should be listed"),
        )
        .long_about("Lists client connections that are authenticated with a specific username")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            CONNECTION_GUIDE_URL
        ));

    [
        close_connection,
        close_user_connections,
        list_cmd,
        list_user_connections_cmd,
    ]
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

fn definitions_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let export_cmd = Command::new("export")
        .about("Export cluster-wide definitions")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEFINITION_GUIDE_URL
        ))
        .arg(
            Arg::new("file")
                .group("output")
                .long("file")
                .help("output file path")
                .required(false)
                .default_value("-")
                .conflicts_with("stdout"),
        )
        .arg(
            Arg::new("stdout")
                .group("output")
                .long("stdout")
                .help("print result to the standard output stream")
                .required(false)
                .num_args(0)
                .action(ArgAction::SetTrue)
                .conflicts_with("file"),
        )
        .arg(
            Arg::new("transformations")
                .long("transformations")
                .short('t')
                .long_help(
                    r#"
A comma-separated list of names of the definition transformations to apply.

Supported transformations:

 * prepare_for_quorum_queue_migration
 * strip_cmq_keys_from_policies
 * drop_empty_policies
 * obfuscate_usernames
 * exclude_users
 * exclude_permissions
 * exclude_runtime_parameters
 * exclude_policies
 * no_op

All unknown transformations will be ignored (will be replaced with a `no_op`).

Examples:

 * --transformations prepare_for_quorum_queue_migration,drop_empty_policies
 * --transformations strip_cmq_keys_from_policies,drop_empty_policies
 * --transformations exclude_users,exclude_permissions
 * --transformations obfuscate_usernames
 * --transformations exclude_runtime_parameters,exclude_policies
 * --transformations no_op
                "#,
                )
                .num_args(1..)
                .value_delimiter(',')
                .action(ArgAction::Append)
                .required(false),
        );

    let export_from_vhost_cmd = Command::new("export_from_vhost")
        .about("Export definitions of a specific virtual host")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEFINITION_GUIDE_URL
        ))
        .arg(
            Arg::new("file")
                .group("output")
                .long("file")
                .help("output file path")
                .required(false)
                .default_value("-")
                .conflicts_with("stdout"),
        )
        .arg(
            Arg::new("stdout")
                .group("output")
                .long("stdout")
                .help("print result to the standard output stream")
                .required(false)
                .num_args(0)
                .action(ArgAction::SetTrue)
                .conflicts_with("file"),
        )
        .arg(
            Arg::new("transformations")
                .long("transformations")
                .short('t')
                .long_help(
                    r#"
A comma-separated list of names of the definition transformations to apply.

Supported transformations:

 * prepare_for_quorum_queue_migration
 * strip_cmq_keys_from_policies
 * drop_empty_policies
 * no_op

All unknown transformations will be ignored (will be replaced with a `no_op`).

Examples:

 * --transformations prepare_for_quorum_queue_migration,drop_empty_policies
 * --transformations strip_cmq_keys_from_policies,drop_empty_policies
 * --transformations no_op
                "#,
                )
                .num_args(1..)
                .value_delimiter(',')
                .action(ArgAction::Append)
                .required(false),
        );

    let import_cmd = Command::new("import")
        .about("Import cluster-wide definitions (of multiple virtual hosts)")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEFINITION_GUIDE_URL
        ))
        .arg(
            Arg::new("file")
                .group("input")
                .long("file")
                .help("cluster-wide definitions JSON file path; mutually exclusive with --stdin")
                .required(true)
                .conflicts_with("stdin"),
        )
        .arg(
            Arg::new("stdin")
                .group("input")
                .long("stdin")
                .help("read input JSON from the standard input stream, mutually exclusive with --file")
                .required(false)
                .num_args(0)
                .action(ArgAction::SetTrue)
                .conflicts_with("file"),
        );

    let import_into_vhost_cmd = Command::new("import_into_vhost")
        .about("Import a virtual host-specific definitions file into a virtual host")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEFINITION_GUIDE_URL
        ))
        .arg(
            Arg::new("file")
                .group("input")
                .long("file")
                .help("cluster-wide definitions JSON file path; mutually exclusive with --stdin")
                .required(true)
                .conflicts_with("stdin"),
        )
        .arg(
            Arg::new("stdin")
                .group("input")
                .long("stdin")
                .help("read input JSON from the standard input stream, mutually exclusive with --file")
                .required(false)
                .num_args(0)
                .action(ArgAction::SetTrue)
                .conflicts_with("file"),
        );

    [
        export_cmd,
        export_from_vhost_cmd,
        import_cmd,
        import_into_vhost_cmd,
    ]
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

fn exchanges_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let bind_cmd = Command::new("bind")
        .about("Creates a binding between a source exchange and a destination (a queue or an exchange)")
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
        );
    let declare_cmd = Command::new("declare")
        .about("Declares an exchange")
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
        );
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);
    let delete_cmd = Command::new("delete")
        .about("Deletes an exchange")
        .arg(
            Arg::new("name")
                .long("name")
                .help("exchange name")
                .required(true),
        )
        .arg(idempotently_arg.clone());
    let list_cmd = Command::new("list").long_about("Lists exchanges");
    let unbind_cmd = Command::new("unbind")
        .about("Deletes a binding")
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
        )
        .arg(idempotently_arg.clone());
    [bind_cmd, declare_cmd, delete_cmd, list_cmd, unbind_cmd]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}
fn export_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let definitions = Command::new("definitions")
        .about("Export cluster-wide definitions")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEFINITION_GUIDE_URL
        ))
        .arg(
            Arg::new("file")
                .group("output")
                .long("file")
                .help("output file path")
                .required(false)
                .default_value("-")
                .conflicts_with("stdout"),
        )
        .arg(
            Arg::new("stdout")
                .group("output")
                .long("stdout")
                .help("print result to the standard output stream")
                .required(false)
                .num_args(0)
                .action(ArgAction::SetTrue)
                .conflicts_with("file"),
        )
        .arg(
            Arg::new("transformations")
                .long("transformations")
                .short('t')
                .long_help(
                    r#"
A comma-separated list of names of the definition transformations to apply.

Supported transformations:

 * no_op
 * prepare_for_quorum_queue_migration
 * strip_cmq_keys_from_policies
 * drop_empty_policies
 * obfuscate_usernames
 * exclude_users
 * exclude_permissions
 * exclude_runtime_parameters
 * exclude_policies

Examples:

 * --transformations prepare_for_quorum_queue_migration,drop_empty_policies
 * --transformations strip_cmq_keys_from_policies,drop_empty_policies
 * --transformations exclude_users,exclude_permissions
 * --transformations obfuscate_usernames
 * --transformations exclude_runtime_parameters,exclude_policies
 * --transformations no_op
                "#,
                )
                .num_args(1..)
                .value_delimiter(',')
                .action(ArgAction::Append)
                .required(false),
        );
    [definitions]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

fn import_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    [Command::new("definitions")
        .about("Prefer 'definitions import'")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEFINITION_GUIDE_URL
        ))
        .arg(
            Arg::new("file")
                .long("file")
                .help("JSON file with definitions")
                .required(true),
        )]
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

pub fn feature_flags_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let list_cmd = Command::new("list")
        .long_about("Lists feature flags and their cluster state")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            FEATURE_FLAG_GUIDE_URL
        ));

    let enable_cmd = Command::new("enable")
        .long_about("Enables a feature flag")
        .after_help(color_print::cformat!(
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
        .long_about("Enables all stable feature flags")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            FEATURE_FLAG_GUIDE_URL
        ));

    [list_cmd, enable_cmd, enable_all_cmd]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

pub fn deprecated_features_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let list_cmd = Command::new("list")
        .long_about("Lists deprecated features")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEPRECATED_FEATURE_GUIDE_URL
        ));

    let list_in_use_cmd = Command::new("list_used")
        .long_about("Lists the deprecated features that are found to be in use in the cluster")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            DEPRECATED_FEATURE_GUIDE_URL
        ));

    [list_cmd, list_in_use_cmd]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

pub fn plugins_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let list_all_cmd = Command::new("list_all")
        .about("Lists plugins across all cluster nodes")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            PLUGIN_GUIDE_URL
        ));

    let list_on_node_cmd = Command::new("list_on_node")
        .about("Lists plugins enabled on a specific node")
        .arg(
            Arg::new("node")
                .long("node")
                .help("target node, must be a cluster member")
                .required(true),
        )
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            PLUGIN_GUIDE_URL
        ));

    [
        list_all_cmd,
        list_on_node_cmd,
    ]
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

pub fn nodes_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let list_cmd = Command::new("list")
        .long_about("Lists cluster nodes")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            CLUSTERING_GUIDE_URL
        ));

    let memory_breakdown_in_bytes_cmd = Command::new("memory_breakdown_in_bytes")
        .about("Provides a memory footprint breakdown (in bytes) for the target node")
        .arg(
            Arg::new("node")
                .long("node")
                .help("target node, must be a cluster member")
                .required(true),
        )
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            MEMORY_FOOTPRINT_GUIDE_URL
        ));

    let memory_breakdown_in_percent_cmd = Command::new("memory_breakdown_in_percent")
        .about("Provides a memory footprint breakdown (in percent) for the target node")
        .arg(
            Arg::new("node")
                .long("node")
                .help("target node, must be a cluster member")
                .required(true),
        )
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            MEMORY_FOOTPRINT_GUIDE_URL
        ));

    [
        list_cmd,
        memory_breakdown_in_percent_cmd,
        memory_breakdown_in_bytes_cmd,
    ]
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

pub fn vhosts_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let list_cmd = Command::new("list")
        .long_about("Lists virtual hosts")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            VIRTUAL_HOST_GUIDE_URL
        ));

    let declare_cmd = Command::new("declare")
        .about("Creates a virtual host")
        .after_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", VIRTUAL_HOST_GUIDE_URL))
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
        );

    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);
    let delete_cmd = Command::new("delete")
        .about("Deletes a virtual host")
        .arg(
            Arg::new("name")
                .long("name")
                .help("virtual host")
                .required(true),
        )
        .arg(idempotently_arg.clone());

    let bulk_delete_cmd = Command::new("delete_multiple")
        .about(color_print::cstr!("<bold><red>DANGER ZONE.</red></bold> Deletes multiple virtual hosts at once using a name matching pattern"))
        .after_help(color_print::cformat!("<bold>Doc guide:</bold>: {}", VIRTUAL_HOST_GUIDE_URL))
        .arg(
            Arg::new("name_pattern")
                .long("name-pattern")
                .help("a regular expression that will be used to match virtual host names")
                .required(true),
        )
        .arg(
            Arg::new("approve")
                .long("approve")
                .action(ArgAction::SetTrue)
                .help("this operation is very destructive and requires an explicit approval")
                .required(false),
        )
        .arg(
            Arg::new("dry_run")
                .long("dry-run")
                .action(ArgAction::SetTrue)
                .help("show what would be deleted without performing the actual deletion")
                .required(false),
        )
        .arg(idempotently_arg.clone());
    let enable_deletion_protection_cmd = Command::new("enable_deletion_protection")
        .about("Enables deletion protection for a virtual host")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            VHOST_DELETION_PROTECTION_GUIDE_URL
        ))
        .arg(
            Arg::new("name")
                .long("name")
                .help("virtual host name")
                .required(true),
        );
    let disable_deletion_protection_cmd = Command::new("disable_deletion_protection")
        .about("Disables deletion protection for a virtual host")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            VHOST_DELETION_PROTECTION_GUIDE_URL
        ))
        .arg(
            Arg::new("name")
                .long("name")
                .help("virtual host name")
                .required(true),
        );

    [
        list_cmd,
        declare_cmd,
        delete_cmd,
        bulk_delete_cmd,
        enable_deletion_protection_cmd,
        disable_deletion_protection_cmd,
    ]
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

pub fn users_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let declare_cmd = Command::new("declare")
        .about("Creates a user")
        .arg(
            Arg::new("name")
                .long("name")
                .help("username")
                .required(true),
        )
        .arg(
            Arg::new("password_hash")
                .help(color_print::cformat!(
                    "salted password hash, see {}",
                    PASSWORD_GUIDE_URL
                ))
                .long("password-hash")
                .required(false)
                .default_value(""),
        )
        .arg(
            Arg::new("password")
                .long("password")
                .help(color_print::cformat!(
                    "prefer providing a hash, see {}",
                    PASSWORD_GUIDE_URL
                ))
                .required(false)
                .default_value(""),
        )
        .arg(
            Arg::new("hashing_algorithm")
                .long("hashing-algorithm")
                .required(false)
                .conflicts_with("password_hash")
                .requires("password")
                .value_parser(value_parser!(HashingAlgorithm))
                .default_value("SHA256")
                .help("The hashing algorithm to use: SHA256 or SHA512"),
        )
        .arg(
            Arg::new("tags")
                .long("tags")
                .help("a list of comma-separated tags")
                .default_value(""),
        );
    let list_cmd = Command::new("list").long_about("Lists users in the internal database");
    let permissions_cmd = Command::new("permissions")
        .long_about("Lists user permissions")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            ACCESS_CONTROL_GUIDE_URL
        ));
    let connections_cmd = Command::new("connections")
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .required(true)
                .help("Name of the user whose connections should be listed"),
        )
        .long_about("Lists client connections that authenticated with a specific username")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            CONNECTION_GUIDE_URL
        ));
    let limits_cmd = Command::new("limits")
        .arg(
            Arg::new("user")
                .long("user")
                .help("username")
                .required(false),
        )
        .long_about("Lists per-user (resource) limits")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            USER_LIMIT_GUIDE_URL
        ));

    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);
    let delete_cmd = Command::new("delete")
        .about("Deletes a user")
        .arg(
            Arg::new("name")
                .long("name")
                .help("username")
                .required(true),
        )
        .arg(idempotently_arg.clone());

    [
        connections_cmd,
        declare_cmd,
        delete_cmd,
        limits_cmd,
        list_cmd,
        permissions_cmd,
    ]
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

pub fn passwords_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let hash_password = Command::new("salt_and_hash")
        .arg(
            Arg::new("password")
                .required(true)
                .help("A cleartext password value to hash"),
        )
        .arg(
            Arg::new("hashing_algorithm")
                .long("hashing-algorithm")
                .required(false)
                .value_parser(value_parser!(HashingAlgorithm))
                .default_value("SHA256")
                .help("The hashing algorithm to use: SHA256 or SHA512"),
        );

    [hash_password]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

pub fn permissions_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);

    let list_cmd = Command::new("list")
        .long_about("Lists user permissions")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            ACCESS_CONTROL_GUIDE_URL
        ));

    let declare_cmd = Command::new("declare")
        .about("grants permissions to a user")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            ACCESS_CONTROL_GUIDE_URL
        ))
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
        );

    let delete_cmd = Command::new("delete")
        .about("Revokes user permissions to a given vhost")
        .arg(
            Arg::new("user")
                .long("user")
                .help("username")
                .required(true),
        )
        .arg(idempotently_arg.clone());

    [list_cmd, declare_cmd, delete_cmd]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

pub fn user_limits_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let list_cmd = Command::new("list")
        .long_about("Lists per-user (resource) limits")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            USER_LIMIT_GUIDE_URL
        ))
        .arg(
            Arg::new("user")
                .long("user")
                .help("username")
                .required(false),
        );

    let declare_cmd = Command::new("declare")
        .about("Set a user limit")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            USER_LIMIT_GUIDE_URL
        ))
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
        );

    let delete_cmd = Command::new("delete")
        .about("Clears a user limit")
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
        );

    [list_cmd, declare_cmd, delete_cmd]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

pub fn vhost_limits_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let list_cmd = Command::new("list")
        .long_about("Lists virtual host (resource) limits")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            VIRTUAL_HOST_GUIDE_URL
        ));

    let declare_cmd = Command::new("declare")
        .about("Set a vhost limit")
        .after_help(color_print::cformat!(
            "<bold>Doc guide:</bold>: {}",
            VIRTUAL_HOST_LIMIT_GUIDE_URL
        ))
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
        );

    let delete_cmd = Command::new("delete").about("delete a vhost limit").arg(
        Arg::new("name")
            .long("name")
            .help("limit name (eg. max-connections, max-queues)")
            .required(true),
    );

    [list_cmd, declare_cmd, delete_cmd]
        .into_iter()
        .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
        .collect()
}

pub fn publish_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    [Command::new("message")
        .about(color_print::cstr!("Publishes (<red>inefficiently</red>) message(s) to a queue or a stream. <bold><red>Only suitable for development and test environments</red></bold>. Prefer messaging or streaming protocol clients!"))
        .after_help(color_print::cformat!("<bold>Doc guide</bold>: {}", PUBLISHER_GUIDE_URL))
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
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

pub fn get_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    [Command::new("messages")
        .about(color_print::cstr!("Fetches (via <red>polling, very inefficiently</red>) message(s) from a queue. <bold><red>Only suitable for development and test environments</red></bold>"))
        .after_help(color_print::cformat!("<bold>Doc guide</bold>: {}", POLLING_CONSUMER_GUIDE_URL))
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
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

pub fn shovel_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);

    let list_all_cmd = Command::new("list_all")
        .long_about("Lists shovels in all virtual hosts")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            SHOVEL_GUIDE_URL
        ));

    let list_cmd = Command::new("list")
        .long_about("Lists shovels in a specific virtual host")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            SHOVEL_GUIDE_URL
        ));

    let declare_091_cmd = Command::new("declare_amqp091")
        .long_about(
            "Declares a dynamic shovel that uses AMQP 0-9-1 for both source and destination",
        )
        .after_help(color_print::cformat!(
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
                .value_parser(value_parser!(MessageTransferAcknowledgementMode)),
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
                .value_parser(value_parser!(u32)),
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
        .after_help(color_print::cformat!(
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
                .value_parser(value_parser!(MessageTransferAcknowledgementMode)),
        )
        .arg(Arg::new("source_address").long("source-address"))
        .arg(Arg::new("destination_address").long("destination-address"))
        .arg(
            Arg::new("reconnect_delay")
                .long("reconnect-delay")
                .default_value("5")
                .value_parser(value_parser!(u32)),
        );

    let delete_cmd = Command::new("delete")
        .long_about("Deletes a dynamic shovel")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            SHOVEL_GUIDE_URL
        ))
        .arg(
            Arg::new("name")
                .long("name")
                .help("shovel name (identifier)")
                .required(true),
        )
        .arg(idempotently_arg.clone());

    let disable_tls_peer_verification_cmd = Command::new("disable_tls_peer_verification_for_all_source_uris")
        // shorter, displayed in the shovels group's help
        .about(color_print::cstr!("<bold><red>Use only in case of emergency</red></bold>. Disables TLS peer verification for all shovels."))
        // longer, displayed in the command's help
        .long_about(color_print::cstr!("<bold><red>Use only in case of emergency</red></bold>. Disables TLS peer verification for all shovels by updating their source and destination URIs' 'verify' parameter."))
        .after_help(color_print::cformat!(
            r#"<bold>Doc guides</bold>:

 * {}
 * {}
 * {}"#,
            SHOVEL_GUIDE_URL,
            TLS_GUIDE_URL,
            "https://www.rabbitmq.com/docs/shovel#tls-connections"
        ));

    let disable_tls_peer_verification_dest_cmd = Command::new("disable_tls_peer_verification_for_all_destination_uris")
        .about(color_print::cstr!("<bold><red>Use only in case of emergency</red></bold>. Disables TLS peer verification for all shovel destination URIs."))
        .long_about(color_print::cstr!("<bold><red>Use only in case of emergency</red></bold>. Disables TLS peer verification for all shovel destination URIs by updating their 'verify' parameter."))
        .after_help(color_print::cformat!(
            r#"<bold>Doc guides</bold>:

 * {}
 * {}
 * {}"#,
            SHOVEL_GUIDE_URL,
            TLS_GUIDE_URL,
            "https://www.rabbitmq.com/docs/shovel#tls-connections"
        ));

    let enable_tls_peer_verification_source_cmd = Command::new("enable_tls_peer_verification_for_all_source_uris")
        .about("Enables TLS peer verification for all shovel source URIs with provided [RabbitMQ node-local] certificate paths.")
        .long_about("Enables TLS peer verification for all shovel source URIs by updating their 'verify' parameter and adding [RabbitMQ node-local] certificate and private key file paths.")
        .arg(
            Arg::new("node_local_ca_certificate_bundle_path")
                .long("node-local-ca-certificate-bundle-path")
                .help("Path to the CA certificate bundle file on the target RabbitMQ node(s)")
                .required(true)
                .value_name("PATH")
        )
        .arg(
            Arg::new("node_local_client_certificate_file_path")
                .long("node-local-client-certificate-file-path")
                .help("Path to the client certificate file on the target RabbitMQ node(s)")
                .required(true)
                .value_name("PATH")
        )
        .arg(
            Arg::new("node_local_client_private_key_file_path")
                .long("node-local-client-private-key-file-path")
                .help("Path to the client private key file on the target RabbitMQ node(s)")
                .required(true)
                .value_name("PATH")
        )
        .after_help(color_print::cformat!(
            r#"<bold>Doc guides</bold>:

 * {}
 * {}
 * {}"#,
            SHOVEL_GUIDE_URL,
            TLS_GUIDE_URL,
            "https://www.rabbitmq.com/docs/shovel#tls-connections"
        ));

    let enable_tls_peer_verification_dest_cmd = Command::new("enable_tls_peer_verification_for_all_destination_uris")
        .about("Enables TLS peer verification for all shovel destination URIs with provided [RabbitMQ node-local] certificate paths.")
        .long_about("Enables TLS peer verification for all shovel destination URIs by updating their 'verify' parameter and adding [RabbitMQ node-local] certificate and private key file paths.")
        .arg(
            Arg::new("node_local_ca_certificate_bundle_path")
                .long("node-local-ca-certificate-bundle-path")
                .help("Path to the CA certificate bundle file on the target RabbitMQ node(s)")
                .required(true)
                .value_name("PATH")
        )
        .arg(
            Arg::new("node_local_client_certificate_file_path")
                .long("node-local-client-certificate-file-path")
                .help("Path to the client certificate file on the target RabbitMQ node(s)")
                .required(true)
                .value_name("PATH")
        )
        .arg(
            Arg::new("node_local_client_private_key_file_path")
                .long("node-local-client-private-key-file-path")
                .help("Path to the client private key file on the target RabbitMQ node(s)")
                .required(true)
                .value_name("PATH")
        )
        .after_help(color_print::cformat!(
            r#"<bold>Doc guides</bold>:

 * {}
 * {}
 * {}"#,
            SHOVEL_GUIDE_URL,
            TLS_GUIDE_URL,
            "https://www.rabbitmq.com/docs/shovel#tls-connections"
        ));

    [
        list_all_cmd,
        list_cmd,
        declare_091_cmd,
        declare_10_cmd,
        delete_cmd,
        disable_tls_peer_verification_cmd,
        disable_tls_peer_verification_dest_cmd,
        enable_tls_peer_verification_source_cmd,
        enable_tls_peer_verification_dest_cmd,
    ]
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}

fn federation_subcommands(pre_flight_settings: PreFlightSettings) -> Vec<Command> {
    let idempotently_arg = Arg::new("idempotently")
        .long("idempotently")
        .value_parser(value_parser!(bool))
        .action(ArgAction::SetTrue)
        .help("do not consider 404 Not Found API responses to be errors")
        .required(false);

    let list_all_upstreams = Command::new("list_all_upstreams")
        .long_about("Lists federation upstreams in all virtual hosts")
        .after_help(color_print::cformat!(
            r#"<bold>Doc guides</bold>:

 * {}
 * {}
 * {}"#,
            FEDERATION_GUIDE_URL,
            FEDERATED_EXCHANGES_GUIDE_URL,
            FEDERATED_QUEUES_GUIDE_URL
        ));

    let declare_upstream = Command::new("declare_upstream")
        .long_about("Declares a federation upstream to be used with both exchange and queue federation")
        .after_help(color_print::cformat!(
                    r#"<bold>Doc guides</bold>:

 * {}
 * {}
 * {}
 * {}"#,
                    FEDERATION_GUIDE_URL,
                    FEDERATED_EXCHANGES_GUIDE_URL,
                    FEDERATED_QUEUES_GUIDE_URL,
                    FEDERATION_REFERENCE_URL
                ))
        .arg(
            Arg::new("name")
                .long("name")
                .help("upstream name (identifier)")
                .required(true),
        )
        .arg(
            Arg::new("uri")
                .long("uri")
                .help("the URI to use to connect to this upstream")
                .required(true),
        )
        .arg(
            Arg::new("reconnect_delay")
                .long("reconnect-delay")
                .default_value("5")
                .value_parser(value_parser!(u32))
                .help("Reconnection delay in seconds")
        )
        .arg(
            Arg::new("trust_user_id")
                .long("trust-user-id")
                .default_value("true")
                .value_parser(value_parser!(bool))
                .help("If set to true, federation will pass through any validated user-id from the upstream, even though it cannot validate it")
        )
        .arg(
            Arg::new("prefetch_count")
                .long("prefetch-count")
                .default_value("1000")
                .value_parser(value_parser!(u32))
                .help("The prefetch value to use with internal consumers")
                .value_parser(value_parser!(u32))
        )
        .arg(
            Arg::new("ack_mode")
                .long("ack-mode")
                .value_parser(value_parser!(MessageTransferAcknowledgementMode))
                .help("Accepted values are: on-confirm, on-publish, no-ack")
                .default_value("on-confirm"),
        )
        .arg(
            Arg::new("queue_name")
                .long("queue-name")
                .help("queue federation: the queue name to use on the upstream. Defaults to the federated queue name")
        )
        .arg(
            Arg::new("consumer_tag")
                .long("consumer-tag")
                .help("Custom consumer tag to use for the internal federation consumer")
                .requires("queue_name")
        )
        .arg(
            Arg::new("exchange_name")
                .long("exchange-name")
                .help("exchange federation: the exchange name to use on the upstream. Defaults to the federated exchange name")
        )
        .arg(
            Arg::new("queue_type")
                .long("queue-type")
                .help("exchange federation: the type of the internal queue to use")
                .default_value(DEFAULT_QUEUE_TYPE)
                .help(color_print::cformat!("default queue type, one of: <bold>classic</bold>, <bright-blue>quorum</bright-blue>, <bright-magenta>stream</bright-magenta>"))
        )
        .arg(
            Arg::new("max_hops")
                .long("max_hops")
                .default_value("1")
                .value_parser(value_parser!(u8))
        )
        .arg(
            Arg::new("bind_nowait")
                .long("bind-using-nowait")
                .default_value("false")
                .value_parser(value_parser!(bool))
        )
        .arg(
            Arg::new("resource_cleanup_mode")
                .long("resource-cleanup-mode")
                .default_value("default")
                .value_parser(value_parser!(FederationResourceCleanupMode))
        )
        .arg(
            Arg::new("channel_use_mode")
                .long("channel-use-mode")
                .default_value("multiple")
                .value_parser(value_parser!(ChannelUseMode))
        )
        .arg(
            Arg::new("ttl")
                .long("ttl")
                .long_help("exchange federation: the TTL to apply to the internal queue")
                .value_parser(value_parser!(u32))
        )
        .arg(
            Arg::new("message_ttl")
                .long("message-ttl")
                .long_help("exchange federation: the message TTL to use with the internal queue")
                .value_parser(value_parser!(u32))
        );

    let declare_upstream_for_queue_federation = Command::new("declare_upstream_for_queues")
        .long_about("Declares an upstream that will be used only for queue federation")
        .after_help(color_print::cformat!(
                    r#"<bold>Doc guides</bold>:

 * {}
 * {}
 * {}"#,
                    FEDERATION_GUIDE_URL,
                    FEDERATED_QUEUES_GUIDE_URL,
                    FEDERATION_REFERENCE_URL
                ))
        .arg(
            Arg::new("name")
                .long("name")
                .help("upstream name (identifier)")
                .required(true),
        )
        .arg(
            Arg::new("uri")
                .long("uri")
                .help("the URI to use to connect to this upstream")
                .required(true),
        )
        .arg(
            Arg::new("reconnect_delay")
                .long("reconnect-delay")
                .default_value("5")
                .value_parser(value_parser!(u32))
                .help("Reconnection delay in seconds")
        )
        .arg(
            Arg::new("trust_user_id")
                .long("trust-user-id")
                .default_value("true")
                .value_parser(value_parser!(bool))
                .help("If set to true, federation will pass through any validated user-id from the upstream, even though it cannot validate it")
        )
        .arg(
            Arg::new("prefetch_count")
                .long("prefetch-count")
                .default_value("1000")
                .value_parser(value_parser!(u32))
                .help("The prefetch value to use with internal consumers")
        )
        .arg(
            Arg::new("ack_mode")
                .long("ack-mode")
                .value_parser(value_parser!(MessageTransferAcknowledgementMode))
                .help("Accepted values are: on-confirm, on-publish, no-ack")
                .default_value("on-confirm"),
        )
        .arg(
            Arg::new("bind_nowait")
                .long("bind-using-nowait")
                .default_value("false")
                .value_parser(value_parser!(bool))
        )
        .arg(
            Arg::new("channel_use_mode")
                .long("channel-use-mode")
                .default_value("multiple")
                .value_parser(value_parser!(ChannelUseMode))
        )
        .arg(
            Arg::new("queue_name")
                .long("queue-name")
                .help("queue federation: the queue name to use on the upstream. Defaults to the federated queue name")
        )
        .arg(
            Arg::new("consumer_tag")
                .long("consumer-tag")
                .help("Custom consumer tag to use for the internal federation consumer")
                .requires("queue_name")
        );

    let declare_upstream_for_exchange_federation = Command::new("declare_upstream_for_exchanges")
        .long_about("Declares an upstream that will be used only for exchange federation")
        .after_help(color_print::cformat!(
                    r#"<bold>Doc guides</bold>:

 * {}
 * {}
 * {}"#,
                    FEDERATION_GUIDE_URL,
                    FEDERATED_EXCHANGES_GUIDE_URL,
                    FEDERATION_REFERENCE_URL
                ))
        .arg(
            Arg::new("name")
                .long("name")
                .help("upstream name (identifier)")
                .required(true),
        )
        .arg(
            Arg::new("uri")
                .long("uri")
                .help("the URI to use to connect to this upstream")
                .required(true),
        )
        .arg(
            Arg::new("reconnect_delay")
                .long("reconnect-delay")
                .default_value("5")
                .value_parser(value_parser!(u32))
                .help("Reconnection delay in seconds")
        )
        .arg(
            Arg::new("trust_user_id")
                .long("trust-user-id")
                .default_value("true")
                .value_parser(value_parser!(bool))
                .help("If set to true, federation will pass through any validated user-id from the upstream, even though it cannot validate it")
        )
        .arg(
            Arg::new("prefetch_count")
                .long("prefetch-count")
                .default_value("1000")
                .value_parser(value_parser!(u32))
                .help("The prefetch value to use with internal consumers")
        )
        .arg(
            Arg::new("ack_mode")
                .long("ack-mode")
                .value_parser(value_parser!(MessageTransferAcknowledgementMode))
                .help("Accepted values are: on-confirm, on-publish, no-ack")
                .default_value("on-confirm"),
        )
        .arg(
            Arg::new("exchange_name")
                .long("exchange-name")
                .help("exchange federation: the exchange name to use on the upstream. Defaults to the federated exchange name")
        )
        .arg(
            Arg::new("queue_type")
                .long("queue-type")
                .help("exchange federation: the type of the internal queue to use")
                .default_value(DEFAULT_QUEUE_TYPE)
                .help(color_print::cformat!("default queue type, one of: <bold>classic</bold>, <bright-blue>quorum</bright-blue>, <bright-magenta>stream</bright-magenta>"))
        )
        .arg(
            Arg::new("max_hops")
                .long("max-hops")
                .default_value("1")
                .value_parser(value_parser!(u8))
        )
        .arg(
            Arg::new("resource_cleanup_mode")
                .long("resource-cleanup-mode")
                .default_value("default")
                .value_parser(value_parser!(FederationResourceCleanupMode))
        )
        .arg(
            Arg::new("channel_use_mode")
                .long("channel-use-mode")
                .default_value("multiple")
                .value_parser(value_parser!(ChannelUseMode))
        )
        .arg(
            Arg::new("bind_nowait")
                .long("bind-using-nowait")
                .default_value("false")
                .value_parser(value_parser!(bool))
        )
        .arg(
            Arg::new("ttl")
                .long("ttl")
                .long_help("exchange federation: the TTL to apply to the internal queue")
                .value_parser(value_parser!(u32))
        )
        .arg(
            Arg::new("message_ttl")
                .long("message-ttl")
                .long_help("exchange federation: the message TTL to use with the internal queue")
                .value_parser(value_parser!(u32))
        );

    let delete_upstream = Command::new("delete_upstream")
        .long_about("Declares a federation upstream")
        .after_help(color_print::cformat!(
            "<bold>Doc guide</bold>: {}",
            FEDERATION_GUIDE_URL
        ))
        .arg(
            Arg::new("name")
                .long("name")
                .help("upstream name (identifier)")
                .required(true),
        )
        .arg(idempotently_arg.clone());

    let list_all_links = Command::new("list_all_links")
        .long_about("List federation links in all virtual hosts")
        .after_help(color_print::cformat!(
            r#"<bold>Doc guides</bold>:

 * {}
 * {}
 * {}
 * {}"#,
            FEDERATION_GUIDE_URL,
            FEDERATED_EXCHANGES_GUIDE_URL,
            FEDERATED_QUEUES_GUIDE_URL,
            FEDERATION_REFERENCE_URL
        ));

    let disable_tls_peer_verification_cmd = Command::new("disable_tls_peer_verification_for_all_upstreams")
        // shorter, displayed in the federation group's help
        .about(color_print::cstr!("<bold><red>Use only in case of emergency</red></bold>. Disables TLS peer verification for all federation upstreams."))
        // longer, displayed in the command's help
        .long_about(color_print::cstr!("<bold><red>Use only in case of emergency</red></bold>. Disables TLS peer verification for all federation upstreams by updating their 'verify' parameter."))

        .after_help(color_print::cformat!(
            r#"<bold>Doc guides</bold>:

 * {}
 * {}
 * {}"#,
            FEDERATION_GUIDE_URL,
            TLS_GUIDE_URL,
            "https://www.rabbitmq.com/docs/federation#tls-connections"
        ));

    let enable_tls_peer_verification_cmd = Command::new("enable_tls_peer_verification_for_all_upstreams")
        .about("Enables TLS peer verification for all federation upstreams with provided [RabbitMQ node-local] certificate paths.")
        .long_about("Enables TLS peer verification for all federation upstreams by updating their 'verify' parameter and adding [RabbitMQ node-local] certificate and private key file paths.")
        .arg(
            Arg::new("node_local_ca_certificate_bundle_path")
                .long("node-local-ca-certificate-bundle-path")
                .help("Path to the CA certificate bundle file on the target RabbitMQ node(s)")
                .required(true)
                .value_name("PATH")
        )
        .arg(
            Arg::new("node_local_client_certificate_file_path")
                .long("node-local-client-certificate-file-path")
                .help("Path to the client certificate file on the target RabbitMQ node(s)")
                .required(true)
                .value_name("PATH")
        )
        .arg(
            Arg::new("node_local_client_private_key_file_path")
                .long("node-local-client-private-key-file-path")
                .help("Path to the client private key file on the target RabbitMQ node(s)")
                .required(true)
                .value_name("PATH")
        )
        .after_help(color_print::cformat!(
            r#"<bold>Doc guides</bold>:

 * {}
 * {}
 * {}"#,
            FEDERATION_GUIDE_URL,
            TLS_GUIDE_URL,
            "https://www.rabbitmq.com/docs/federation#tls-connections"
        ));

    [
        list_all_upstreams,
        declare_upstream,
        declare_upstream_for_exchange_federation,
        declare_upstream_for_queue_federation,
        delete_upstream,
        list_all_links,
        disable_tls_peer_verification_cmd,
        enable_tls_peer_verification_cmd,
    ]
    .into_iter()
    .map(|cmd| cmd.infer_long_args(pre_flight_settings.infer_long_options))
    .collect()
}
