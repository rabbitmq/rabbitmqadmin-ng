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
#![allow(clippy::result_large_err)]

use clap::ArgMatches;
use errors::CommandRunError;
use reqwest::Identity;
use std::path::PathBuf;
use std::{fs, process};
use sysexits::ExitCode;

use rustls::pki_types::pem::PemObject;

mod cli;
mod commands;
mod config;
mod constants;
mod errors;
mod output;
mod static_urls;
mod tables;
mod tanzu_cli;
mod tanzu_commands;

use crate::config::SharedSettings;
use crate::constants::{
    DEFAULT_CONFIG_FILE_PATH, DEFAULT_HTTPS_PORT, DEFAULT_NODE_ALIAS, DEFAULT_VHOST,
    TANZU_COMMAND_PREFIX,
};
use crate::output::*;
use rabbitmq_http_client::blocking_api::{Client as GenericAPIClient, ClientBuilder};
use reqwest::blocking::Client as HTTPClient;
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

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

    match build_http_client(&cli, &common_settings) {
        Ok(httpc) => {
            // SharedSettings considers not just one but multiple ways to obtain
            // the value if it wasn't passed on the command line, so these are
            // safe to unwrap()
            let username = common_settings.username.clone().unwrap();
            let password = common_settings.password.clone().unwrap();
            let client = build_rabbitmq_http_api_client(httpc, &endpoint, &username, &password);

            if let Some((first_level, first_level_args)) = cli.subcommand() {
                if let Some((second_level, second_level_args)) = first_level_args.subcommand() {
                    // this is a Tanzu RabbitMQ-specific command, these are grouped under "tanzu"
                    if first_level == TANZU_COMMAND_PREFIX {
                        if let Some((third_level, third_level_args)) =
                            second_level_args.subcommand()
                        {
                            let pair = (second_level, third_level);

                            // let vhost = virtual_host(&common_settings, second_level_args);

                            let mut res_handler =
                                ResultHandler::new(&common_settings, second_level_args);
                            let exit_code = dispatch_tanzu_subcommand(
                                pair,
                                third_level_args,
                                client,
                                &mut res_handler,
                            );

                            process::exit(exit_code.into())
                        }
                    } else {
                        // this is a common (OSS and Tanzu) command
                        let pair = (first_level, second_level);

                        let vhost = virtual_host(&common_settings, second_level_args);

                        let mut res_handler =
                            ResultHandler::new(&common_settings, second_level_args);
                        let exit_code = dispatch_common_subcommand(
                            pair,
                            second_level_args,
                            client,
                            common_settings.endpoint(),
                            vhost,
                            &mut res_handler,
                        );

                        process::exit(exit_code.into())
                    }
                }
            }
        }
        Err(err) => {
            let mut res_handler = ResultHandler::new(&common_settings, &cli);
            res_handler.report_pre_command_run_error(&err);
            let code = res_handler.exit_code.unwrap_or(ExitCode::DataErr);
            process::exit(code.into())
        }
    }
}

fn build_rabbitmq_http_api_client<'a>(
    httpc: HTTPClient,
    endpoint: &'a str,
    username: &'a str,
    password: &'a str,
) -> APIClient<'a> {
    ClientBuilder::new()
        .with_endpoint(endpoint)
        .with_basic_auth_credentials(username, password)
        .with_client(httpc)
        .build()
}

fn build_http_client(
    cli: &ArgMatches,
    common_settings: &SharedSettings,
) -> Result<HTTPClient, CommandRunError> {
    let user_agent = format!("rabbitmqadmin-ng {}", clap::crate_version!());
    if should_use_tls(common_settings) {
        let _ = CryptoProvider::install_default(rustls::crypto::aws_lc_rs::default_provider());

        let ca_cert_pem_file = cli.get_one::<PathBuf>("tls-ca-cert-file");

        let maybe_client_cert_pem_file = cli.get_one::<PathBuf>("tls-cert-file");
        let maybe_client_key_pem_file = cli.get_one::<PathBuf>("tls-key-file");

        let ca_certs = ca_cert_pem_file
            .map(|path| load_certs(&path.to_string_lossy()))
            .unwrap()?;

        let disable_peer_verification = *cli.get_one::<bool>("insecure").unwrap_or(&false);

        let mut builder = HTTPClient::builder()
            .user_agent(user_agent)
            .use_rustls_tls()
            .tls_info(true)
            .tls_sni(true)
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .tls_built_in_root_certs(true)
            .danger_accept_invalid_certs(disable_peer_verification)
            .danger_accept_invalid_hostnames(disable_peer_verification);

        // --tls-ca-cert-file
        let mut store = rustls::RootCertStore::empty();
        for c in ca_certs {
            store.add(c).map_err(|err| {
                let readable_path = maybe_client_cert_pem_file
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                CommandRunError::CertificateStoreRejectedCertificate {
                    local_path: readable_path,
                    cause: err,
                }
            })?;
        }

        // --tls-cert-file, --tls-key-file
        if maybe_client_cert_pem_file.is_some() && maybe_client_key_pem_file.is_some() {
            let client_cert_pem_file = maybe_client_cert_pem_file.unwrap();
            let client_key_pem_file = maybe_client_key_pem_file.unwrap();

            let client_cert = fs::read(client_cert_pem_file)?;
            let client_key = fs::read(client_key_pem_file)?;

            let concatenated = [&client_cert[..], &client_key[..]].concat();
            let client_id = Identity::from_pem(&concatenated).map_err(|err| {
                let readable_path = maybe_client_key_pem_file
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                CommandRunError::CertificateFileCouldNotBeLoaded1 {
                    local_path: readable_path,
                    cause: err,
                }
            })?;

            builder = builder.identity(client_id);
        }

        Ok(builder.build().unwrap())
    } else {
        Ok(HTTPClient::builder()
            .user_agent(user_agent)
            .build()
            .unwrap())
    }
}

type CertificateChain = Vec<CertificateDer<'static>>;

fn load_certs(filename: &str) -> Result<CertificateChain, CommandRunError> {
    let results = CertificateDer::pem_file_iter(filename)
        .map_err(|err| {
            let readable_path = filename.to_string();
            CommandRunError::CertificateFileCouldNotBeLoaded2 {
                local_path: readable_path,
                cause: err,
            }
        })
        .unwrap();
    let certs = results.map(|it| it.unwrap()).collect::<CertificateChain>();
    Ok(certs)
}

#[allow(dead_code)]
fn load_private_key(filename: &str) -> Result<PrivateKeyDer<'static>, CommandRunError> {
    PrivateKeyDer::from_pem_file(filename).map_err(|err| {
        let readable_path = filename.to_string();
        CommandRunError::CertificateFileCouldNotBeLoaded2 {
            local_path: readable_path,
            cause: err,
        }
    })
}

fn dispatch_common_subcommand(
    pair: (&str, &str),
    second_level_args: &ArgMatches,
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
        ("show", "memory_breakdown_in_bytes") => {
            let result = commands::show_memory_breakdown(client, second_level_args);
            res_handler.memory_breakdown_in_bytes_result(result)
        }
        ("show", "memory_breakdown_in_percent") => {
            let result = commands::show_memory_breakdown(client, second_level_args);
            res_handler.memory_breakdown_in_percent_result(result)
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
            let result = commands::list_user_limits(client, second_level_args);
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
            let result = commands::list_parameters(client, &vhost, second_level_args);
            res_handler.tabular_result(result)
        }
        ("list", "exchanges") => {
            let result = commands::list_exchanges(client, &vhost);
            res_handler.tabular_result(result)
        }
        ("list", "feature_flags") => {
            let result = commands::list_feature_flags(client);
            res_handler.tabular_result(result.map(|val| val.0))
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
            let result = commands::declare_vhost(client, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "exchange") => {
            let result = commands::declare_exchange(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "user") => {
            let result = commands::declare_user(client, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "permissions") => {
            let result = commands::declare_permissions(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "permissions") => {
            let result = commands::delete_permissions(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "queue") => {
            let result = commands::declare_queue(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "stream") => {
            let result = commands::declare_stream(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "binding") => {
            let result = commands::declare_binding(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "policy") => {
            let result = commands::declare_policy(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "operator_policy") => {
            let result = commands::declare_operator_policy(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "vhost_limit") => {
            let result = commands::declare_vhost_limit(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "user_limit") => {
            let result = commands::declare_user_limit(client, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "parameter") => {
            let result = commands::declare_parameter(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "vhost") => {
            let result = commands::delete_vhost(client, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("delete", "user") => {
            let result = commands::delete_user(client, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("delete", "exchange") => {
            let result = commands::delete_exchange(client, &vhost, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("delete", "queue") => {
            let result = commands::delete_queue(client, &vhost, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("delete", "stream") => {
            let result = commands::delete_stream(client, &vhost, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("delete", "binding") => {
            let result = commands::delete_binding(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "policy") => {
            let result = commands::delete_policy(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "operator_policy") => {
            let result = commands::delete_operator_policy(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "vhost_limit") => {
            let result = commands::delete_vhost_limit(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "user_limit") => {
            let result = commands::delete_user_limit(client, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "parameter") => {
            let result = commands::delete_parameter(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("purge", "queue") => {
            let result = commands::purge_queue(client, &vhost, second_level_args);
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
        ("health_check", "port_listener") => {
            let result = commands::health_check_port_listener(client, second_level_args);
            res_handler.health_check_result(result);
        }
        ("health_check", "protocol_listener") => {
            let result = commands::health_check_protocol_listener(client, second_level_args);
            res_handler.health_check_result(result);
        }
        ("rebalance", "queues") => {
            let result = commands::rebalance_queues(client);
            res_handler.no_output_on_success(result);
        }
        ("close", "connection") => {
            let result = commands::close_connection(client, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("definitions", "export") => {
            let result = commands::export_definitions(client, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("definitions", "import") => {
            let result = commands::import_definitions(client, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("export", "definitions") => {
            let result = commands::export_definitions(client, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("import", "definitions") => {
            let result = commands::import_definitions(client, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("feature_flags", "list") => {
            let result = commands::list_feature_flags(client);
            res_handler.tabular_result(result.map(|val| val.0))
        }
        ("feature_flags", "enable") => {
            let result = commands::enable_feature_flag(client, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("feature_flags", "enable_all") => {
            let result = commands::enable_all_stable_feature_flags(client);
            res_handler.no_output_on_success(result);
        }
        ("deprecated_features", "list") => {
            let result = commands::list_deprecated_features(client);
            res_handler.tabular_result(result.map(|val| val.0))
        }
        ("deprecated_features", "list_used") => {
            let result = commands::list_deprecated_features_in_use(client);
            res_handler.tabular_result(result.map(|val| val.0))
        }
        ("publish", "message") => {
            let result = commands::publish_message(client, &vhost, second_level_args);
            res_handler.single_value_result(result)
        }
        ("get", "messages") => {
            let result = commands::get_messages(client, &vhost, second_level_args);
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

fn dispatch_tanzu_subcommand(
    pair: (&str, &str),
    third_level_args: &ArgMatches,
    client: APIClient<'_>,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match &pair {
        ("sds", "status") => {
            let result = tanzu_commands::sds_status(client, third_level_args);
            res_handler.schema_definition_sync_status_result(result)
        }
        ("sds", "enable") => {
            let result = tanzu_commands::sds_enable(client, third_level_args);
            res_handler.no_output_on_success(result)
        }
        ("sds", "disable") => {
            let result = tanzu_commands::sds_disable(client, third_level_args);
            res_handler.no_output_on_success(result)
        }
        ("wsr", "status") => {
            let result = tanzu_commands::wsr_status(client);
            res_handler.warm_standby_replication_status_result(result)
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
