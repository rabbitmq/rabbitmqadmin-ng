// Copyright (C) 2023-2024 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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
use clap::ArgMatches;
use errors::CommandRunError;
use reqwest::Certificate;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process;
use sysexits::ExitCode;

mod cli;
mod commands;
mod config;
mod constants;
mod errors;
mod output;
mod static_urls;
mod tables;

use crate::config::SharedSettings;
use crate::constants::{
    DEFAULT_CONFIG_FILE_PATH, DEFAULT_HTTPS_PORT, DEFAULT_NODE_ALIAS, DEFAULT_VHOST,
};
use crate::output::*;
use rabbitmq_http_client::blocking_api::{Client as GenericAPIClient, ClientBuilder};
use reqwest::blocking::Client as HTTPClient;

type APIClient<'a> = GenericAPIClient<&'a str, &'a str, &'a str>;

fn main() {
    let parser = cli::parser();
    let cli = parser.get_matches();
    let default_config_file_path = PathBuf::from(DEFAULT_CONFIG_FILE_PATH);
    let config_file_path = cli
        .get_one::<PathBuf>("config_file_path")
        .cloned()
        .unwrap_or(PathBuf::from(DEFAULT_CONFIG_FILE_PATH));
    let uses_default_config_file_path = config_file_path == default_config_file_path;

    // config file entries are historically called nodes
    let node_alias = cli
        .get_one::<String>("node_alias")
        .cloned()
        .or(Some(DEFAULT_NODE_ALIAS.to_string()));

    let cf_ss = SharedSettings::from_config_file(&config_file_path, node_alias.clone());
    // If the default config file path is used and the function above
    // reports that it is not found, continue. Otherwise exit.
    if cf_ss.is_err() && !uses_default_config_file_path {
        eprintln!(
            "Encountered an error when trying to load configuration for node alias '{}' in configuration file '{}'",
            &node_alias.unwrap(),
            config_file_path.to_str().unwrap()
        );
        eprintln!("Underlying error: {}", cf_ss.unwrap_err());
        process::exit(ExitCode::DataErr.into())
    }
    let common_settings = if let Ok(val) = cf_ss {
        SharedSettings::from_args_with_defaults(&cli, &val)
    } else {
        SharedSettings::from_args(&cli)
    };
    let endpoint = common_settings.endpoint();
    let disable_peer_verification = *cli.get_one::<bool>("insecure").unwrap_or(&false);

    let user_agent = format!("rabbitmqadmin-ng {}", clap::crate_version!());
    let httpc = if should_use_tls(&common_settings) {
        let mut cert = None;
        if let Some(pem_file) = cli.get_one::<PathBuf>("tls-ca-cert-file") {
            let mut file = File::open(pem_file).unwrap_or_else(|err| {
                eprintln!("unable to open {}: {}", pem_file.to_string_lossy(), err);
                process::exit(1);
            });

            let mut pem = Vec::new();
            if let Err(err) = file.read_to_end(&mut pem) {
                eprintln!("unable to read {}: {}", pem_file.to_string_lossy(), err);
                process::exit(1);
            }

            let res = match Certificate::from_pem(&pem) {
                Ok(val) => val,
                Err(err) => {
                    eprintln!(
                        "{} doesn't seem to be a valid PEM file: {}",
                        pem_file.to_string_lossy(),
                        err
                    );
                    process::exit(1);
                }
            };

            cert = Some(res)
        }

        let mut b = HTTPClient::builder()
            .user_agent(user_agent)
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .danger_accept_invalid_certs(disable_peer_verification)
            .danger_accept_invalid_hostnames(disable_peer_verification);

        if cert.is_some() {
            b = b.add_root_certificate(cert.unwrap());
        }

        b.build()
    } else {
        HTTPClient::builder().user_agent(user_agent).build()
    }
    .unwrap();

    let username = common_settings.username.clone().unwrap();
    let password = common_settings.password.clone().unwrap();
    let client = ClientBuilder::new()
        .with_endpoint(endpoint.as_str())
        .with_basic_auth_credentials(username.as_str(), password.as_str())
        .with_client(httpc)
        .build();

    if let Some((verb, group_args)) = cli.subcommand() {
        if let Some((kind, command_args)) = group_args.subcommand() {
            let pair = (verb, kind);

            let vhost = virtual_host(&common_settings, command_args);

            let mut res_handler = ResultHandler::new(&common_settings, command_args);
            let exit_code = dispatch_subcommand(
                pair,
                command_args,
                client,
                common_settings.endpoint(),
                vhost,
                &mut res_handler,
            );

            process::exit(exit_code.into())
        }
    }
}

fn dispatch_subcommand(
    pair: (&str, &str),
    command_args: &ArgMatches,
    client: APIClient<'_>,
    endpoint: String,
    vhost: String,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match &pair {
        ("show", "overview") => {
            let result = commands::show_overview(client);
            res_handler.show_overview(result)
        }
        ("show", "churn") => {
            let result = commands::show_overview(client);
            res_handler.show_churn(result)
        }
        ("show", "endpoint") => {
            println!("Using endpoint: {}", endpoint);
            res_handler.no_output_on_success(Ok(()))
        }

        ("list", "nodes") => {
            let result = commands::list_nodes(client);
            res_handler.tabular_result(result)
        }
        ("list", "vhosts") => {
            let result = commands::list_vhosts(client);
            res_handler.tabular_result(result)
        }
        ("list", "vhost_limits") => {
            let result = commands::list_vhost_limits(client, &vhost);
            res_handler.tabular_result(result)
        }
        ("list", "user_limits") => {
            let result = commands::list_user_limits(client, command_args);
            res_handler.tabular_result(result)
        }
        ("list", "users") => {
            let result = commands::list_users(client);
            res_handler.tabular_result(result)
        }
        ("list", "connections") => {
            let result = commands::list_connections(client);
            res_handler.tabular_result(result)
        }
        ("list", "channels") => {
            let result = commands::list_channels(client);
            res_handler.tabular_result(result)
        }
        ("list", "consumers") => {
            let result = commands::list_consumers(client);
            res_handler.tabular_result(result)
        }
        ("list", "policies") => {
            let result = commands::list_policies(client);
            res_handler.tabular_result(result)
        }
        ("list", "operator_policies") => {
            let result = commands::list_operator_policies(client);
            res_handler.tabular_result(result)
        }
        ("list", "queues") => {
            let result = commands::list_queues(client, &vhost);
            res_handler.tabular_result(result)
        }
        ("list", "bindings") => {
            let result = commands::list_bindings(client);
            res_handler.tabular_result(result)
        }
        ("list", "permissions") => {
            let result = commands::list_permissions(client);
            res_handler.tabular_result(result)
        }
        ("list", "parameters") => {
            let result = commands::list_parameters(client, &vhost, command_args);
            res_handler.tabular_result(result)
        }
        ("list", "exchanges") => {
            let result = commands::list_exchanges(client, &vhost);
            res_handler.tabular_result(result)
        }
        ("list", "deprecated_features") => {
            let result = commands::list_deprecated_features(client);
            res_handler.tabular_result(result.map(|val| val.0))
        }
        ("list", "deprecated_features_in_use") => {
            let result = commands::list_deprecated_features_in_use(client);
            res_handler.tabular_result(result.map(|val| val.0))
        }
        ("declare", "vhost") => {
            let result = commands::declare_vhost(client, command_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "exchange") => {
            let result = commands::declare_exchange(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "user") => {
            let result = commands::declare_user(client, command_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "permissions") => {
            let result = commands::declare_permissions(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "permissions") => {
            let result = commands::delete_permissions(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "queue") => {
            let result = commands::declare_queue(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "binding") => {
            let result = commands::declare_binding(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "policy") => {
            let result = commands::declare_policy(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "operator_policy") => {
            let result = commands::declare_operator_policy(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "vhost_limit") => {
            let result = commands::declare_vhost_limit(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "user_limit") => {
            let result = commands::declare_user_limit(client, command_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "parameter") => {
            let result = commands::declare_parameter(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "vhost") => {
            let result = commands::delete_vhost(client, command_args);
            res_handler.delete_operation_result(result);
        }
        ("delete", "user") => {
            let result = commands::delete_user(client, command_args);
            res_handler.delete_operation_result(result);
        }
        ("delete", "exchange") => {
            let result = commands::delete_exchange(client, &vhost, command_args);
            res_handler.delete_operation_result(result);
        }
        ("delete", "queue") => {
            let result = commands::delete_queue(client, &vhost, command_args);
            res_handler.delete_operation_result(result);
        }
        ("delete", "binding") => {
            let result = commands::delete_binding(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "policy") => {
            let result = commands::delete_policy(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "operator_policy") => {
            let result = commands::delete_operator_policy(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "vhost_limit") => {
            let result = commands::delete_vhost_limit(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "user_limit") => {
            let result = commands::delete_user_limit(client, command_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "parameter") => {
            let result = commands::delete_parameter(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("purge", "queue") => {
            let result = commands::purge_queue(client, &vhost, command_args);
            res_handler.no_output_on_success(result);
        }
        ("health_check", "local_alarms") => {
            let result = commands::health_check_local_alarms(client);
            res_handler.health_check_result(result);
        }
        ("health_check", "cluster_wide_alarms") => {
            let result = commands::health_check_cluster_wide_alarms(client);
            res_handler.health_check_result(result);
        }
        ("health_check", "node_is_quorum_critical") => {
            let result = commands::health_check_node_is_quorum_critical(client);
            res_handler.health_check_result(result);
        }
        ("rebalance", "queues") => {
            let result = commands::rebalance_queues(client);
            res_handler.no_output_on_success(result);
        }
        ("close", "connection") => {
            let result = commands::close_connection(client, command_args);
            res_handler.no_output_on_success(result);
        }
        ("definitions", "export") => {
            let result = commands::export_definitions(client, command_args);
            res_handler.no_output_on_success(result);
        }
        ("definitions", "import") => {
            let result = commands::import_definitions(client, command_args);
            res_handler.no_output_on_success(result);
        }
        ("export", "definitions") => {
            let result = commands::export_definitions(client, command_args);
            res_handler.no_output_on_success(result);
        }
        ("import", "definitions") => {
            let result = commands::import_definitions(client, command_args);
            res_handler.no_output_on_success(result);
        }
        ("publish", "message") => {
            let result = commands::publish_message(client, &vhost, command_args);
            res_handler.single_value_result(result)
        }
        ("get", "messages") => {
            let result = commands::get_messages(client, &vhost, command_args);
            res_handler.tabular_result(result)
        }
        _ => {
            let error = CommandRunError::UnknownCommandTarget {
                command: pair.0.into(),
                subcommand: pair.1.into(),
            };
            res_handler.report_pre_command_run_error(&error);
        }
    }

    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn should_use_tls(shared_settings: &SharedSettings) -> bool {
    shared_settings.scheme.to_lowercase() == "https"
        || shared_settings.port.unwrap_or(DEFAULT_HTTPS_PORT) == DEFAULT_HTTPS_PORT
}

/// Retrieves a --vhost value, either from global or command-specific arguments
fn virtual_host(shared_settings: &SharedSettings, command_flags: &ArgMatches) -> String {
    // in case a command does not define --vhost
    if command_flags.try_contains_id("vhost").is_ok() {
        // if the command-specific flag is not set to default,
        // use it, otherwise use the global/shared --vhost flag value
        let fallback = String::from(DEFAULT_VHOST);
        let command_vhost: &str = command_flags
            .get_one::<String>("vhost")
            .unwrap_or(&fallback);

        if command_vhost != DEFAULT_VHOST {
            String::from(command_vhost)
        } else {
            shared_settings
                .virtual_host
                .clone()
                .unwrap_or(DEFAULT_VHOST.to_string())
        }
    } else {
        shared_settings
            .virtual_host
            .clone()
            .unwrap_or(DEFAULT_VHOST.to_string())
    }
}
