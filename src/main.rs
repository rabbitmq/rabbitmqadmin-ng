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
#![allow(clippy::unnecessary_unwrap)]
#![allow(clippy::collapsible_if)]

use clap::{ArgMatches, crate_name, crate_version};
use errors::CommandRunError;
use reqwest::{Identity, tls::Version as TlsVersion};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, process};
use sysexits::ExitCode;

use rustls::pki_types::pem::PemObject;

mod cli;
mod commands;
mod config;
mod constants;
mod errors;
mod output;
pub mod pre_flight;
mod static_urls;
mod tables;
mod tanzu_cli;
mod tanzu_commands;

use crate::config::{PreFlightSettings, SharedSettings};
use crate::constants::{
    DEFAULT_CONFIG_FILE_PATH, DEFAULT_HTTPS_PORT, DEFAULT_NODE_ALIAS, DEFAULT_VHOST,
    TANZU_COMMAND_PREFIX,
};
use crate::output::*;
use rabbitmq_http_client::blocking_api::{Client as GenericAPIClient, ClientBuilder};
use rabbitmq_http_client::commons::PolicyTarget;
use reqwest::blocking::Client as HTTPClient;
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

type APIClient = GenericAPIClient<String, String, String>;

type CertificateChain = Vec<CertificateDer<'static>>;

fn main() {
    let pre_flight_settings = if pre_flight::is_non_interactive() {
        PreFlightSettings::non_interactive()
    } else {
        PreFlightSettings {
            infer_subcommands: pre_flight::should_infer_subcommands(),
            infer_long_options: pre_flight::should_infer_long_options(),
        }
    };

    let parser = cli::parser(pre_flight_settings);
    let cli = parser.get_matches();

    let (common_settings, endpoint) = resolve_run_configuration(&cli);

    match configure_http_api_client(&cli, &common_settings, &endpoint.clone()) {
        Ok(client) => {
            let exit_code = dispatch_command(&cli, client, &common_settings);
            process::exit(exit_code.into())
        }
        Err(err) => {
            let mut res_handler = ResultHandler::new(&common_settings, &cli);
            res_handler.report_pre_command_run_error(&err);
            let code = res_handler.exit_code.unwrap_or(ExitCode::DataErr);
            process::exit(code.into())
        }
    }
}

fn resolve_run_configuration(cli: &ArgMatches) -> (SharedSettings, String) {
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
        .or_else(|| Some(DEFAULT_NODE_ALIAS.to_string()));

    // If the default config file path is used and the function above
    // reports that it is not found, continue. Otherwise, exit.
    let cf_ss = SharedSettings::from_config_file(&config_file_path, node_alias.clone());
    if cf_ss.is_err() && !uses_default_config_file_path {
        eprintln!(
            "Encountered an error when trying to load configuration for node alias '{}' in configuration file '{}'",
            &node_alias.unwrap(),
            config_file_path.to_str().unwrap()
        );
        eprintln!("Underlying error: {}", cf_ss.unwrap_err());
        process::exit(ExitCode::DataErr.into())
    }

    let common_settings = cf_ss
        .map(|val| SharedSettings::from_args_with_defaults(cli, &val))
        .unwrap_or_else(|_| SharedSettings::from_args(cli));
    let endpoint = common_settings.endpoint();

    (common_settings, endpoint)
}

fn configure_http_api_client<'a>(
    cli: &'a ArgMatches,
    merged_settings: &'a SharedSettings,
    endpoint: &'a str,
) -> Result<APIClient, CommandRunError> {
    let httpc = build_http_client(cli, merged_settings)?;
    // Due to how SharedSettings are computed, these should safe to unwrap()
    let username = merged_settings.username.clone().unwrap();
    let password = merged_settings.password.clone().unwrap();

    // Extract timeout from CLI arguments (default is 60 seconds)
    let timeout_secs = cli.get_one::<u64>("timeout").copied().unwrap_or(60);
    let timeout = Duration::from_secs(timeout_secs);

    let client = build_rabbitmq_http_api_client(
        httpc,
        endpoint.to_owned(),
        username.clone(),
        password.clone(),
        timeout,
    );
    Ok(client)
}

fn dispatch_command(
    cli: &ArgMatches,
    client: APIClient,
    merged_settings: &SharedSettings,
) -> ExitCode {
    if let Some((first_level, first_level_args)) = cli.subcommand() {
        if let Some((second_level, second_level_args)) = first_level_args.subcommand() {
            return if first_level == TANZU_COMMAND_PREFIX {
                // this is a Tanzu RabbitMQ-specific command, these are grouped under "tanzu"
                if let Some((third_level, third_level_args)) = second_level_args.subcommand() {
                    let pair = (second_level, third_level);
                    let mut res_handler = ResultHandler::new(merged_settings, second_level_args);
                    dispatch_tanzu_subcommand(pair, third_level_args, client, &mut res_handler)
                } else {
                    ExitCode::Usage
                }
            } else {
                // this is a common (OSS and Tanzu) command
                let pair = (first_level, second_level);
                let vhost = virtual_host(merged_settings, second_level_args);
                let mut res_handler = ResultHandler::new(merged_settings, second_level_args);
                dispatch_common_subcommand(
                    pair,
                    second_level_args,
                    client,
                    merged_settings.endpoint(),
                    vhost,
                    &mut res_handler,
                )
            };
        }
    }
    ExitCode::Usage
}

fn build_rabbitmq_http_api_client(
    httpc: HTTPClient,
    endpoint: String,
    username: String,
    password: String,
    timeout: Duration,
) -> APIClient {
    ClientBuilder::new()
        .with_endpoint(endpoint)
        .with_basic_auth_credentials(username, password)
        .with_client(httpc)
        .with_request_timeout(timeout)
        .build()
}

fn build_http_client(
    cli: &ArgMatches,
    common_settings: &SharedSettings,
) -> Result<HTTPClient, CommandRunError> {
    let user_agent = format!("{} {}", crate_name!(), crate_version!());
    if should_use_tls(common_settings) {
        let _ = CryptoProvider::install_default(rustls::crypto::aws_lc_rs::default_provider());

        let ca_cert_pem_file = common_settings.ca_certificate_bundle_path.clone();
        let maybe_client_cert_pem_file = common_settings.client_certificate_file_path.clone();
        let maybe_client_key_pem_file = common_settings.client_private_key_file_path.clone();

        let ca_certs_path_opt = ca_cert_pem_file.clone();

        let disable_peer_verification = *cli.get_one::<bool>("insecure").unwrap_or(&false);

        let mut builder = HTTPClient::builder()
            .user_agent(user_agent)
            .use_rustls_tls()
            .tls_info(true)
            .tls_sni(true)
            .min_tls_version(TlsVersion::TLS_1_2)
            .tls_built_in_native_certs(true)
            .tls_built_in_root_certs(true)
            .danger_accept_invalid_certs(disable_peer_verification)
            .danger_accept_invalid_hostnames(disable_peer_verification);

        // local certificate store
        let mut store = rustls::RootCertStore::empty();

        if let Some(ca_certs_path) = ca_certs_path_opt {
            let ca_certs_path_str = ca_certs_path.to_string_lossy();

            // Load CA certificates with improved error handling
            let ca_certs = load_certs(&ca_certs_path_str).map_err(|err| {
                // Add context about this being a CA certificate bundle
                match err {
                    CommandRunError::CertificateFileNotFound { local_path } => {
                        CommandRunError::CertificateFileNotFound {
                            local_path: format!("CA certificate bundle at {}", local_path),
                        }
                    }
                    CommandRunError::CertificateFileEmpty { local_path } => {
                        CommandRunError::CertificateFileEmpty {
                            local_path: format!("CA certificate bundle at {}", local_path),
                        }
                    }
                    CommandRunError::CertificateFileInvalidPem {
                        local_path,
                        details,
                    } => CommandRunError::CertificateFileInvalidPem {
                        local_path: format!("CA certificate bundle at {}", local_path),
                        details,
                    },
                    other => other,
                }
            })?;

            for (index, cert) in ca_certs.into_iter().enumerate() {
                store.add(cert).map_err(|err| {
                    let readable_path = ca_cert_pem_file
                        .clone()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();

                    // Provide more context about which certificate failed
                    let detailed_path = if index == 0 {
                        readable_path
                    } else {
                        format!("{} (certificate #{} in bundle)", readable_path, index + 1)
                    };

                    CommandRunError::CertificateStoreRejectedCertificate {
                        local_path: detailed_path,
                        cause: err,
                    }
                })?;
            }
        }

        // --tls-cert-file, --tls-key-file
        if maybe_client_cert_pem_file.is_some() && maybe_client_key_pem_file.is_some() {
            let client_cert_pem_file = maybe_client_cert_pem_file.clone().unwrap();
            let client_key_pem_file = maybe_client_key_pem_file.clone().unwrap();

            let cert_path = client_cert_pem_file.to_string_lossy().to_string();
            let key_path = client_key_pem_file.to_string_lossy().to_string();

            // Validate both files exist and are readable
            validate_certificate_file(&cert_path)?;
            validate_certificate_file(&key_path)?;

            let client_cert = read_pem_file(&client_cert_pem_file, &cert_path)?;
            let client_key = read_pem_file(&client_key_pem_file, &key_path)?;

            let concatenated = [&client_cert[..], &client_key[..]].concat();
            let client_id = Identity::from_pem(&concatenated).map_err(|err| {
                // Try to determine if it's a key/cert mismatch or other issue
                if err.to_string().contains("private key")
                    || err.to_string().contains("certificate")
                {
                    CommandRunError::CertificateKeyMismatch {
                        cert_path: cert_path.clone(),
                        key_path: key_path.clone(),
                    }
                } else {
                    CommandRunError::CertificateFileCouldNotBeLoaded1 {
                        local_path: cert_path,
                        cause: err,
                    }
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

fn read_pem_file(buf: &PathBuf, file_path: &str) -> Result<Vec<u8>, CommandRunError> {
    fs::read(buf).map_err(|err| CommandRunError::CertificateFileCouldNotBeLoaded2 {
        local_path: file_path.to_owned(),
        cause: rustls::pki_types::pem::Error::Io(err),
    })
}

fn validate_certificate_file(path: &str) -> Result<(), CommandRunError> {
    let path_buf = Path::new(path);

    if !path_buf.exists() {
        return Err(CommandRunError::CertificateFileNotFound {
            local_path: path.to_string(),
        });
    }

    if !path_buf.is_file() {
        return Err(CommandRunError::CertificateFileNotFound {
            local_path: path.to_string(),
        });
    }

    // Check if file is readable
    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.len() == 0 {
                return Err(CommandRunError::CertificateFileEmpty {
                    local_path: path.to_string(),
                });
            }
        }
        Err(_) => {
            return Err(CommandRunError::CertificateFileNotFound {
                local_path: path.to_string(),
            });
        }
    }

    Ok(())
}

fn load_certs(filename: &str) -> Result<CertificateChain, CommandRunError> {
    validate_certificate_file(filename)?;

    let results = CertificateDer::pem_file_iter(filename).map_err(|err| {
        let readable_path = filename.to_string();
        let details = match err {
            rustls::pki_types::pem::Error::NoItemsFound => {
                "Invalid PEM format or structure".to_string()
            }
            rustls::pki_types::pem::Error::IllegalSectionStart { .. } => {
                "Invalid PEM format or structure".to_string()
            }
            rustls::pki_types::pem::Error::MissingSectionEnd { .. } => {
                "Invalid PEM format or structure".to_string()
            }
            _ => format!("Failed to load a PEM file at {}: {}", filename, err),
        };
        CommandRunError::CertificateFileInvalidPem {
            local_path: readable_path,
            details,
        }
    })?;

    let certs = results
        .map(|result| {
            result.map_err(|err| CommandRunError::CertificateFileInvalidPem {
                local_path: filename.to_string(),
                details: format!("Failed to parse certificate: {}", err),
            })
        })
        .collect::<Result<CertificateChain, CommandRunError>>()?;

    if certs.is_empty() {
        return Err(CommandRunError::CertificateFileEmpty {
            local_path: filename.to_string(),
        });
    }

    Ok(certs)
}

#[allow(dead_code)]
fn load_private_key(filename: &str) -> Result<PrivateKeyDer<'static>, CommandRunError> {
    validate_certificate_file(filename)?;

    PrivateKeyDer::from_pem_file(filename).map_err(|err| {
        let readable_path = filename.to_string();
        match err {
            rustls::pki_types::pem::Error::NoItemsFound => {
                CommandRunError::CertificateFileInvalidPem {
                    local_path: readable_path,
                    details: "Invalid PEM format in private key file".to_string(),
                }
            }
            _ => CommandRunError::PrivateKeyFileUnsupported {
                local_path: readable_path,
            },
        }
    })
}

fn dispatch_common_subcommand(
    pair: (&str, &str),
    second_level_args: &ArgMatches,
    client: APIClient,
    endpoint: String,
    vhost: String,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match &pair {
        ("auth_attempts", "stats") => {
            let result = commands::list_auth_attempts(client, second_level_args);
            res_handler.tabular_result(result)
        }
        ("bindings", "declare") => {
            let result = commands::declare_binding(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("bindings", "delete") => {
            let result = commands::delete_binding(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("bindings", "list") => {
            let result = commands::list_bindings(client);
            res_handler.tabular_result(result)
        }
        ("channels", "list") => {
            let result = commands::list_channels(client);
            res_handler.tabular_result(result)
        }
        ("close", "connection") => {
            let result = commands::close_connection(client, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("close", "user_connections") => {
            let result =
                commands::close_user_connections(client, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("connections", "close") => {
            let result = commands::close_connection(client, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("connections", "close_of_user") => {
            let result =
                commands::close_user_connections(client, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("connections", "list") => {
            let result = commands::list_connections(client, second_level_args);
            res_handler.tabular_result(result)
        }
        ("connections", "list_of_user") => {
            let result = commands::list_user_connections(client, second_level_args);
            res_handler.tabular_result(result)
        }
        ("declare", "binding") => {
            let result = commands::declare_binding(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "exchange") => {
            let result = commands::declare_exchange(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "operator_policy") => {
            let result = commands::declare_operator_policy(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "parameter") => {
            let result = commands::declare_parameter(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("declare", "permissions") => {
            let result = commands::declare_permissions(client, &vhost, second_level_args)
                .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("declare", "policy") => {
            let result = commands::declare_policy(client, &vhost, second_level_args);
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
        ("declare", "user") => {
            let result = commands::declare_user(client, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("declare", "user_limit") => {
            let result =
                commands::declare_user_limit(client, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("declare", "vhost") => {
            let result = commands::declare_vhost(client, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("declare", "vhost_limit") => {
            let result = commands::declare_vhost_limit(client, &vhost, second_level_args)
                .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("definitions", "export") => {
            let result = commands::export_cluster_wide_definitions(client, second_level_args)
                .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("definitions", "export_from_vhost") => {
            let result = commands::export_vhost_definitions(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("definitions", "import") => {
            let result = commands::import_definitions(client, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("definitions", "import_into_vhost") => {
            let result = commands::import_vhost_definitions(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "binding") => {
            let result = commands::delete_binding(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("delete", "exchange") => {
            let result = commands::delete_exchange(client, &vhost, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("delete", "operator_policy") => {
            let result = commands::delete_operator_policy(client, &vhost, second_level_args)
                .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("delete", "parameter") => {
            let result =
                commands::delete_parameter(client, &vhost, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("delete", "permissions") => {
            let result =
                commands::delete_permissions(client, &vhost, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("delete", "policy") => {
            let result =
                commands::delete_policy(client, &vhost, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("delete", "queue") => {
            let result = commands::delete_queue(client, &vhost, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("delete", "shovel") => {
            let result =
                commands::delete_shovel(client, &vhost, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("delete", "stream") => {
            let result = commands::delete_stream(client, &vhost, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("delete", "user") => {
            let result = commands::delete_user(client, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("delete", "user_limit") => {
            let result = commands::delete_user_limit(client, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("delete", "vhost") => {
            let result = commands::delete_vhost(client, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("delete", "vhost_limit") => {
            let result =
                commands::delete_vhost_limit(client, &vhost, second_level_args).map_err(Into::into);
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
        ("export", "definitions") => {
            let result = commands::export_cluster_wide_definitions(client, second_level_args)
                .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("exchanges", "bind") => {
            let result = commands::declare_binding(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("exchanges", "declare") => {
            let result = commands::declare_exchange(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("exchanges", "delete") => {
            let result = commands::delete_exchange(client, &vhost, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("exchanges", "list") => {
            let result = commands::list_exchanges(client, &vhost);
            res_handler.tabular_result(result)
        }
        ("exchanges", "unbind") => {
            let result = commands::delete_binding(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("feature_flags", "enable") => {
            let result =
                commands::enable_feature_flag(client, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("feature_flags", "enable_all") => {
            let result = commands::enable_all_stable_feature_flags(client).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("feature_flags", "list") => {
            let result = commands::list_feature_flags(client);
            res_handler.tabular_result(result.map(|val| val.0))
        }
        ("federation", "declare_upstream") => {
            let result = commands::declare_federation_upstream(client, &vhost, second_level_args)
                .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("federation", "declare_upstream_for_exchanges") => {
            let result = commands::declare_federation_upstream_for_exchange_federation(
                client,
                &vhost,
                second_level_args,
            )
            .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("federation", "declare_upstream_for_queues") => {
            let result = commands::declare_federation_upstream_for_queue_federation(
                client,
                &vhost,
                second_level_args,
            )
            .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("federation", "delete_upstream") => {
            let result = commands::delete_federation_upstream(client, &vhost, second_level_args)
                .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("federation", "list_all_links") => {
            let result = commands::list_federation_links(client);
            res_handler.tabular_result(result)
        }
        ("federation", "list_all_upstreams") => {
            let result = commands::list_federation_upstreams(client);
            res_handler.tabular_result(result)
        }
        ("federation", "disable_tls_peer_verification_for_all_upstreams") => {
            let mut prog_rep = res_handler.instantiate_progress_reporter();
            let result = commands::disable_tls_peer_verification_for_all_federation_upstreams(
                client,
                prog_rep.as_mut(),
            );
            res_handler.no_output_on_success(result);
        }
        ("federation", "enable_tls_peer_verification_for_all_upstreams") => {
            let mut prog_rep = res_handler.instantiate_progress_reporter();
            let result = commands::enable_tls_peer_verification_for_all_federation_upstreams(
                client,
                second_level_args,
                prog_rep.as_mut(),
            );
            res_handler.no_output_on_success(result);
        }
        ("get", "messages") => {
            let result = commands::get_messages(client, &vhost, second_level_args);
            res_handler.tabular_result(result)
        }
        ("global_parameters", "clear") => {
            let result =
                commands::delete_global_parameter(client, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("global_parameters", "list") => {
            let result = commands::list_global_parameters(client);
            res_handler.tabular_result(result)
        }
        ("global_parameters", "set") => {
            let result = commands::declare_global_parameter(client, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("health_check", "cluster_wide_alarms") => {
            let result = commands::health_check_cluster_wide_alarms(client);
            res_handler.health_check_result(result);
        }
        ("health_check", "local_alarms") => {
            let result = commands::health_check_local_alarms(client);
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
        ("import", "definitions") => {
            let result = commands::import_definitions(client, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("list", "bindings") => {
            let result = commands::list_bindings(client);
            res_handler.tabular_result(result)
        }
        ("list", "channels") => {
            let result = commands::list_channels(client);
            res_handler.tabular_result(result)
        }
        ("list", "connections") => {
            let result = commands::list_connections(client, second_level_args);
            res_handler.tabular_result(result)
        }
        ("list", "consumers") => {
            let result = commands::list_consumers(client);
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
        ("list", "exchanges") => {
            let result = commands::list_exchanges(client, &vhost);
            res_handler.tabular_result(result)
        }
        ("list", "feature_flags") => {
            let result = commands::list_feature_flags(client);
            res_handler.tabular_result(result.map(|val| val.0))
        }
        ("list", "nodes") => {
            let result = commands::list_nodes(client);
            res_handler.tabular_result(result)
        }
        ("list", "operator_policies") => {
            let result = commands::list_operator_policies(client);
            res_handler.tabular_result(result)
        }
        ("list", "parameters") => {
            let result = commands::list_parameters(client, &vhost, second_level_args);
            res_handler.tabular_result(result)
        }
        ("list", "permissions") => {
            let result = commands::list_permissions(client);
            res_handler.tabular_result(result)
        }
        ("list", "policies") => {
            let result = commands::list_policies(client);
            res_handler.tabular_result(result)
        }
        ("list", "queues") => {
            let result = commands::list_queues(client, &vhost, second_level_args);
            res_handler.tabular_result(result)
        }
        ("list", "user_connections") => {
            let result = commands::list_user_connections(client, second_level_args);
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
        ("list", "vhost_limits") => {
            let result = commands::list_vhost_limits(client, &vhost);
            res_handler.tabular_result(result)
        }
        ("list", "vhosts") => {
            let result = commands::list_vhosts(client);
            res_handler.tabular_result(result)
        }
        ("nodes", "list") => {
            let result = commands::list_nodes(client);
            res_handler.tabular_result(result)
        }
        ("nodes", "memory_breakdown_in_bytes") => {
            let result = commands::show_memory_breakdown(client, second_level_args);
            res_handler.memory_breakdown_in_bytes_result(result)
        }
        ("nodes", "memory_breakdown_in_percent") => {
            let result = commands::show_memory_breakdown(client, second_level_args);
            res_handler.memory_breakdown_in_percent_result(result)
        }
        ("plugins", "list_all") => {
            let result = commands::list_plugins_across_cluster(client);
            res_handler.tabular_result(result)
        }
        ("plugins", "list_on_node") => {
            let result = commands::list_plugins_on_node(client, second_level_args);
            res_handler.tabular_result(result)
        }
        ("operator_policies", "declare") => {
            let result = commands::declare_operator_policy(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("operator_policies", "delete") => {
            let result = commands::delete_operator_policy(client, &vhost, second_level_args)
                .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("operator_policies", "delete_definition_keys") => {
            let result =
                commands::delete_operator_policy_definition_keys(client, &vhost, second_level_args)
                    .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("operator_policies", "delete_definition_keys_from_all_in") => {
            let result = commands::delete_operator_policy_definition_keys_in(
                client,
                &vhost,
                second_level_args,
            )
            .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("operator_policies", "list") => {
            let result = commands::list_operator_policies(client);
            res_handler.tabular_result(result)
        }
        ("operator_policies", "list_in") => {
            let typ_opt = second_level_args
                .get_one::<PolicyTarget>("apply_to")
                .cloned();
            let result = match typ_opt {
                None => commands::list_operator_policies_in(client, &vhost),
                Some(typ) => {
                    commands::list_operator_policies_in_and_applying_to(client, &vhost, typ)
                }
            };
            res_handler.tabular_result(result)
        }
        ("operator_policies", "list_matching_object") => {
            let name = second_level_args
                .get_one::<String>("name")
                .cloned()
                .unwrap();
            let typ = second_level_args
                .get_one::<PolicyTarget>("type")
                .cloned()
                .unwrap();
            let result = commands::list_matching_operator_policies_in(client, &vhost, &name, typ);
            res_handler.tabular_result(result)
        }
        ("operator_policies", "patch") => {
            let result =
                commands::patch_operator_policy_definition(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("operator_policies", "update_definition") => {
            let result =
                commands::update_operator_policy_definition(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("operator_policies", "update_definitions_of_all_in") => {
            let result = commands::update_all_operator_policy_definitions_in(
                client,
                &vhost,
                second_level_args,
            );
            res_handler.no_output_on_success(result);
        }
        ("parameters", "clear") => {
            let result =
                commands::delete_parameter(client, &vhost, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("parameters", "list_all") => {
            let result = commands::list_all_parameters(client);
            res_handler.tabular_result(result)
        }
        ("parameters", "list") => {
            let result = commands::list_parameters(client, &vhost, second_level_args);
            res_handler.tabular_result(result)
        }
        ("parameters", "list_in") => {
            let result =
                commands::list_parameters_of_component_in(client, &vhost, second_level_args);
            res_handler.tabular_result(result)
        }
        ("parameters", "set") => {
            let result = commands::declare_parameter(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("passwords", "salt_and_hash") => {
            let result = commands::salt_and_hash_password(second_level_args);
            res_handler.show_salted_and_hashed_value(result)
        }
        ("permissions", "list") => {
            let result = commands::list_permissions(client);
            res_handler.tabular_result(result)
        }
        ("permissions", "declare") => {
            let result = commands::declare_permissions(client, &vhost, second_level_args)
                .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("permissions", "delete") => {
            let result =
                commands::delete_permissions(client, &vhost, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("policies", "declare") => {
            let result = commands::declare_policy(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("policies", "declare_override") => {
            let result = commands::declare_policy_override(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("policies", "declare_blanket") => {
            let result = commands::declare_blanket_policy(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("policies", "delete") => {
            let result =
                commands::delete_policy(client, &vhost, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("policies", "delete_definition_keys") => {
            let result = commands::delete_policy_definition_keys(client, &vhost, second_level_args)
                .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("policies", "delete_definition_keys_from_all_in") => {
            let result =
                commands::delete_policy_definition_keys_in(client, &vhost, second_level_args)
                    .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("policies", "list") => {
            let result = commands::list_policies(client);
            res_handler.tabular_result(result)
        }
        ("policies", "list_conflicting") => {
            let result = commands::list_policies_with_conflicting_priorities(client);
            res_handler.tabular_result(result)
        }
        ("policies", "list_conflicting_in") => {
            let result = commands::list_policies_with_conflicting_priorities_in(client, &vhost);
            res_handler.tabular_result(result)
        }
        ("policies", "list_in") => {
            let typ_opt = second_level_args
                .get_one::<PolicyTarget>("apply_to")
                .cloned();
            let result = match typ_opt {
                None => commands::list_policies_in(client, &vhost),
                Some(typ) => commands::list_policies_in_and_applying_to(client, &vhost, typ),
            };
            res_handler.tabular_result(result)
        }
        ("policies", "list_matching_object") => {
            let name = second_level_args
                .get_one::<String>("name")
                .cloned()
                .unwrap();
            let typ = second_level_args
                .get_one::<PolicyTarget>("type")
                .cloned()
                .unwrap();
            let result = commands::list_matching_policies_in(client, &vhost, &name, typ);
            res_handler.tabular_result(result)
        }
        ("policies", "patch") => {
            let result = commands::patch_policy_definition(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("policies", "update_definition") => {
            let result = commands::update_policy_definition(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("policies", "update_definitions_of_all_in") => {
            let result =
                commands::update_all_policy_definitions_in(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("publish", "message") => {
            let result = commands::publish_message(client, &vhost, second_level_args);
            res_handler.single_value_output_with_result(result)
        }
        ("purge", "queue") => {
            let result =
                commands::purge_queue(client, &vhost, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("queues", "declare") => {
            let result = commands::declare_queue(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("queues", "delete") => {
            let result = commands::delete_queue(client, &vhost, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("queues", "list") => {
            let result = commands::list_queues(client, &vhost, second_level_args);
            res_handler.tabular_result(result)
        }
        ("queues", "purge") => {
            let result =
                commands::purge_queue(client, &vhost, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("queues", "rebalance") => {
            let result = commands::rebalance_queues(client).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("rebalance", "queues") => {
            let result = commands::rebalance_queues(client).map_err(Into::into);
            res_handler.no_output_on_success(result);
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
        ("show", "overview") => {
            let result = commands::show_overview(client);
            res_handler.show_overview(result)
        }
        ("shovels", "declare_amqp091") => {
            let source_queue = second_level_args.get_one::<String>("source_queue").cloned();
            let source_exchange = second_level_args
                .get_one::<String>("source_exchange")
                .cloned();

            let destination_queue = second_level_args
                .get_one::<String>("destination_queue")
                .cloned();
            let destination_exchange = second_level_args
                .get_one::<String>("destination_exchange")
                .cloned();

            if source_queue.is_none() && source_exchange.is_none() {
                let err = CommandRunError::MissingOptions {
                    message: "either --source-queue or --source-exchange must be provided"
                        .to_string(),
                };

                res_handler.report_pre_command_run_error(&err)
            } else if destination_queue.is_none() && destination_exchange.is_none() {
                let err = CommandRunError::MissingOptions {
                    message:
                        "either --destination-queue or --destination-exchange must be provided"
                            .to_string(),
                };

                res_handler.report_pre_command_run_error(&err)
            } else {
                let result = commands::declare_amqp091_shovel(client, &vhost, second_level_args)
                    .map_err(Into::into);
                res_handler.no_output_on_success(result);
            }
        }
        ("shovels", "declare_amqp10") => {
            let result = commands::declare_amqp10_shovel(client, &vhost, second_level_args)
                .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("shovels", "delete") => {
            let result =
                commands::delete_shovel(client, &vhost, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("shovels", "list_all") => {
            let result = commands::list_shovels(client);
            res_handler.tabular_result(result)
        }
        ("shovels", "list") => {
            let result = commands::list_shovels_in(client, &vhost);
            res_handler.tabular_result(result)
        }
        ("shovels", "disable_tls_peer_verification_for_all_source_uris") => {
            let mut prog_rep = res_handler.instantiate_progress_reporter();
            let result = commands::disable_tls_peer_verification_for_all_source_uris(
                client,
                prog_rep.as_mut(),
            );
            res_handler.no_output_on_success(result);
        }
        ("shovels", "disable_tls_peer_verification_for_all_destination_uris") => {
            let mut prog_rep = res_handler.instantiate_progress_reporter();
            let result = commands::disable_tls_peer_verification_for_all_destination_uris(
                client,
                prog_rep.as_mut(),
            );
            res_handler.no_output_on_success(result);
        }
        ("shovels", "enable_tls_peer_verification_for_all_source_uris") => {
            let mut prog_rep = res_handler.instantiate_progress_reporter();
            let result = commands::enable_tls_peer_verification_for_all_source_uris(
                client,
                second_level_args,
                prog_rep.as_mut(),
            );
            res_handler.no_output_on_success(result);
        }
        ("shovels", "enable_tls_peer_verification_for_all_destination_uris") => {
            let mut prog_rep = res_handler.instantiate_progress_reporter();
            let result = commands::enable_tls_peer_verification_for_all_destination_uris(
                client,
                second_level_args,
                prog_rep.as_mut(),
            );
            res_handler.no_output_on_success(result);
        }
        ("streams", "declare") => {
            let result = commands::declare_stream(client, &vhost, second_level_args);
            res_handler.no_output_on_success(result);
        }
        ("streams", "delete") => {
            let result = commands::delete_queue(client, &vhost, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("streams", "list") => {
            let result = commands::list_queues(client, &vhost, second_level_args);
            res_handler.tabular_result(result)
        }
        ("users", "connections") => {
            let result = commands::list_user_connections(client, second_level_args);
            res_handler.tabular_result(result)
        }
        ("users", "declare") => {
            let result = commands::declare_user(client, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("users", "delete") => {
            let result = commands::delete_user(client, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("users", "limits") => {
            let result = commands::list_user_limits(client, second_level_args);
            res_handler.tabular_result(result)
        }
        ("users", "list") => {
            let result = commands::list_users(client);
            res_handler.tabular_result(result)
        }
        ("users", "permissions") => {
            let result = commands::list_permissions(client);
            res_handler.tabular_result(result)
        }
        ("user_limits", "list") => {
            let result = commands::list_user_limits(client, second_level_args);
            res_handler.tabular_result(result)
        }
        ("user_limits", "declare") => {
            let result =
                commands::declare_user_limit(client, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("user_limits", "delete") => {
            let result = commands::delete_user_limit(client, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("vhosts", "declare") => {
            let result = commands::declare_vhost(client, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("vhosts", "delete") => {
            let result = commands::delete_vhost(client, second_level_args);
            res_handler.delete_operation_result(result);
        }
        ("vhosts", "delete_multiple") => {
            let mut prog_rep = res_handler.instantiate_progress_reporter();
            let result =
                commands::delete_multiple_vhosts(client, second_level_args, &mut *prog_rep);
            match result {
                Ok(Some(vhosts)) => {
                    res_handler.tabular_result(Ok(vhosts));
                }
                Ok(None) => {
                    res_handler.no_output_on_success(Ok(()));
                }
                Err(e) => {
                    res_handler.no_output_on_success::<()>(Err(e));
                }
            }
        }
        ("vhosts", "list") => {
            let result = commands::list_vhosts(client);
            res_handler.tabular_result(result)
        }
        ("vhosts", "enable_deletion_protection") => {
            let result = commands::enable_vhost_deletion_protection(client, second_level_args)
                .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("vhosts", "disable_deletion_protection") => {
            let result = commands::disable_vhost_deletion_protection(client, second_level_args)
                .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("vhost_limits", "list") => {
            let result = commands::list_vhost_limits(client, &vhost);
            res_handler.tabular_result(result)
        }
        ("vhost_limits", "declare") => {
            let result = commands::declare_vhost_limit(client, &vhost, second_level_args)
                .map_err(Into::into);
            res_handler.no_output_on_success(result);
        }
        ("vhost_limits", "delete") => {
            let result =
                commands::delete_vhost_limit(client, &vhost, second_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result);
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
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match &pair {
        ("sds", "status_on_node") => {
            let result = tanzu_commands::sds_status_on_node(client, third_level_args);
            res_handler.schema_definition_sync_status_result(result)
        }
        ("sds", "enable_cluster_wide") => {
            let result = tanzu_commands::sds_enable_cluster_wide(client).map_err(Into::into);
            res_handler.no_output_on_success(result)
        }
        ("sds", "disable_cluster_wide") => {
            let result = tanzu_commands::sds_disable_cluster_wide(client).map_err(Into::into);
            res_handler.no_output_on_success(result)
        }
        ("sds", "enable_on_node") => {
            let result =
                tanzu_commands::sds_enable_on_node(client, third_level_args).map_err(Into::into);
            res_handler.no_output_on_success(result)
        }
        ("sds", "disable_on_node") => {
            let result =
                tanzu_commands::sds_disable_on_node(client, third_level_args).map_err(Into::into);
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
    shared_settings.tls
        || shared_settings.scheme.to_lowercase() == "https"
        || shared_settings.port.unwrap_or(DEFAULT_HTTPS_PORT) == DEFAULT_HTTPS_PORT
}

/// Retrieves a --vhost value, either from global or command-specific arguments
fn virtual_host(shared_settings: &SharedSettings, command_flags: &ArgMatches) -> String {
    // in case a command does not define --vhost
    if command_flags.try_contains_id("vhost").is_ok() {
        // if the command-specific flag is not set to default,
        // use it, otherwise use the global/shared --vhost flag value
        let fallback = DEFAULT_VHOST.to_string();
        let command_vhost = command_flags
            .get_one::<String>("vhost")
            .unwrap_or(&fallback);

        if command_vhost != DEFAULT_VHOST {
            command_vhost.clone()
        } else {
            shared_settings
                .virtual_host
                .clone()
                .unwrap_or_else(|| DEFAULT_VHOST.to_string())
        }
    } else {
        shared_settings
            .virtual_host
            .clone()
            .unwrap_or_else(|| DEFAULT_VHOST.to_string())
    }
}
