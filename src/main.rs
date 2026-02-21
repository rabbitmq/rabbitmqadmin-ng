// Copyright (C) 2023-2026 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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

use bel7_cli::generate_completions_to_stdout;
use clap::{ArgMatches, crate_name, crate_version};
use errors::CommandRunError;
use reqwest::{Certificate, Identity, tls::Version as TlsVersion};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use sysexits::ExitCode;

use rustls::pki_types::pem::PemObject;

mod arg_helpers;
mod cli;
mod columns;
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

use crate::cli::CompletionShell;
use crate::config::{PreFlightSettings, SharedSettings};
use crate::constants::{
    DEFAULT_CONFIG_FILE_PATH, DEFAULT_HTTPS_PORT, DEFAULT_NODE_ALIAS, DEFAULT_VHOST,
    TANZU_COMMAND_PREFIX,
};
use crate::output::*;
use rabbitmq_http_client::blocking_api::{
    Client as GenericAPIClient, ClientBuilder, EndpointValidationError,
};
use reqwest::blocking::Client as HTTPClient;
use rustls::crypto::CryptoProvider;
use rustls::pki_types::PrivateKeyDer;

type APIClient = GenericAPIClient<String, String, String>;

fn main() -> ExitCode {
    let pre_flight_settings = if pre_flight::is_non_interactive() {
        PreFlightSettings::non_interactive()
    } else {
        PreFlightSettings {
            infer_subcommands: pre_flight::should_infer_subcommands(),
            infer_long_options: pre_flight::should_infer_long_options(),
        }
    };

    let parser = cli::parser(pre_flight_settings.clone());
    let cli = parser.get_matches();

    // Handle config_file commands before trying to build the API client
    if let Some(("config_file", config_file_args)) = cli.subcommand() {
        return dispatch_config_file_command(&cli, config_file_args);
    }

    // Handle shell commands before trying to build the API client
    if let Some(("shell", shell_args)) = cli.subcommand() {
        return dispatch_shell_command(shell_args, pre_flight_settings);
    }

    let (common_settings, endpoint) = match resolve_run_configuration(&cli) {
        Ok(result) => result,
        Err(code) => return code,
    };

    match configure_http_api_client(&cli, &common_settings, &endpoint.clone()) {
        Ok(client) => dispatch_command(&cli, client, &common_settings),
        Err(err) => {
            let mut res_handler = ResultHandler::new(&common_settings, &cli);
            res_handler.report_pre_command_run_error(&err);
            res_handler.exit_code.unwrap_or(ExitCode::DataErr)
        }
    }
}

fn resolve_run_configuration(cli: &ArgMatches) -> Result<(SharedSettings, String), ExitCode> {
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

    // If the default config file path is used and the function above
    // reports that it is not found, continue. Otherwise, exit.
    let cf_ss = SharedSettings::from_config_file(&config_file_path, node_alias.clone());
    if let Err(e) = &cf_ss
        && !uses_default_config_file_path
    {
        eprintln!(
            "Encountered an error when trying to load configuration for node alias '{}' in configuration file '{}'",
            node_alias.as_deref().unwrap_or("<unknown>"),
            config_file_path.to_str().unwrap_or("<non-UTF-8 path>")
        );
        eprintln!("Underlying error: {}", e);
        return Err(ExitCode::DataErr);
    }

    let common_settings = match cf_ss {
        Ok(val) => SharedSettings::from_args_with_defaults(cli, &val),
        Err(_) => SharedSettings::from_args(cli),
    };

    let common_settings = match common_settings {
        Ok(settings) => settings,
        Err(e) => {
            eprintln!("{}", e);
            return Err(ExitCode::DataErr);
        }
    };

    let endpoint = common_settings.endpoint();

    Ok((common_settings, endpoint))
}

fn configure_http_api_client<'a>(
    cli: &'a ArgMatches,
    merged_settings: &'a SharedSettings,
    endpoint: &'a str,
) -> Result<APIClient, CommandRunError> {
    let httpc = build_http_client(cli, merged_settings)?;
    // Due to how SharedSettings are computed, these should be safe to unwrap()
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
    )?;
    Ok(client)
}

fn dispatch_config_file_command(cli: &ArgMatches, config_file_args: &ArgMatches) -> ExitCode {
    let config_file_path = cli
        .get_one::<PathBuf>("config_file_path")
        .cloned()
        .unwrap_or(PathBuf::from(DEFAULT_CONFIG_FILE_PATH));

    let common_settings = SharedSettings::default();
    let mut res_handler = ResultHandler::new(&common_settings, config_file_args);

    if let Some((subcommand, subcommand_args)) = config_file_args.subcommand() {
        match subcommand {
            "show_path" => {
                let result = commands::config_file_show_path(&config_file_path);
                res_handler.local_tabular_result(result);
            }
            "show" => {
                let reveal_passwords = subcommand_args
                    .get_one::<bool>("reveal_passwords")
                    .copied()
                    .unwrap_or(false);
                let result = commands::config_file_show(&config_file_path, reveal_passwords);
                res_handler.local_tabular_result(result);
            }
            "add_node" => {
                let result = commands::config_file_add_node(&config_file_path, subcommand_args);
                res_handler.local_no_output_on_success(result);
            }
            "update_node" => {
                let result = commands::config_file_update_node(&config_file_path, subcommand_args);
                res_handler.local_no_output_on_success(result);
            }
            "delete_node" => {
                let result = commands::config_file_delete_node(&config_file_path, subcommand_args);
                res_handler.local_no_output_on_success(result);
            }
            _ => return ExitCode::Usage,
        }
    }

    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_shell_command(
    shell_args: &ArgMatches,
    pre_flight_settings: PreFlightSettings,
) -> ExitCode {
    if let Some(("completions", completions_args)) = shell_args.subcommand() {
        let shell = completions_args
            .get_one::<CompletionShell>("shell")
            .copied()
            .unwrap_or_else(CompletionShell::detect);

        let mut cmd = cli::parser(pre_flight_settings);
        generate_completions_to_stdout(shell, &mut cmd, "rabbitmqadmin");
        return ExitCode::Ok;
    }
    ExitCode::Usage
}

fn dispatch_command(
    cli: &ArgMatches,
    client: APIClient,
    merged_settings: &SharedSettings,
) -> ExitCode {
    if let Some((first_level, first_level_args)) = cli.subcommand()
        && let Some((second_level, second_level_args)) = first_level_args.subcommand()
    {
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
    ExitCode::Usage
}

fn build_rabbitmq_http_api_client(
    httpc: HTTPClient,
    endpoint: String,
    username: String,
    password: String,
    timeout: Duration,
) -> Result<APIClient, EndpointValidationError> {
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

        let ca_certs_path_opt = common_settings.ca_certificate_bundle_path.clone();
        let maybe_client_cert_pem_file = common_settings.client_certificate_file_path.as_ref();
        let maybe_client_key_pem_file = common_settings.client_private_key_file_path.as_ref();

        let disable_peer_verification = *cli.get_one::<bool>("insecure").unwrap_or(&false);

        let mut builder = HTTPClient::builder()
            .user_agent(user_agent)
            .tls_backend_rustls()
            .tls_info(true)
            .tls_sni(true)
            .tls_version_min(TlsVersion::TLS_1_2)
            .tls_danger_accept_invalid_certs(disable_peer_verification)
            .tls_danger_accept_invalid_hostnames(disable_peer_verification);

        if let Some(ca_certs_path) = ca_certs_path_opt {
            let ca_certs_path_str = ca_certs_path.to_string_lossy().to_string();
            let cert = load_ca_certificate(&ca_certs_path_str)?;
            builder = builder.tls_certs_only([cert]);
        }

        // --tls-cert-file, --tls-key-file
        if let (Some(client_cert_pem_file), Some(client_key_pem_file)) =
            (maybe_client_cert_pem_file, maybe_client_key_pem_file)
        {
            let cert_path = client_cert_pem_file.to_string_lossy().to_string();
            let key_path = client_key_pem_file.to_string_lossy().to_string();

            // Validate both files exist and are readable
            validate_certificate_file(&cert_path)?;
            validate_certificate_file(&key_path)?;

            let client_cert = read_pem_file(client_cert_pem_file, &cert_path)?;
            let client_key = read_pem_file(client_key_pem_file, &key_path)?;

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

        builder
            .build()
            .map_err(CommandRunError::HttpClientBuildError)
    } else {
        HTTPClient::builder()
            .user_agent(user_agent)
            .build()
            .map_err(CommandRunError::HttpClientBuildError)
    }
}

fn read_pem_file(buf: &PathBuf, file_path: &str) -> Result<Vec<u8>, CommandRunError> {
    fs::read(buf).map_err(|err| CommandRunError::CertificateFileCouldNotBeLoaded2 {
        local_path: file_path.to_owned(),
        cause: rustls::pki_types::pem::Error::Io(err),
    })
}

fn validate_certificate_file(path: &str) -> Result<(), CommandRunError> {
    match fs::metadata(path) {
        Ok(meta) if meta.is_file() && meta.len() > 0 => Ok(()),
        Ok(meta) if meta.is_file() => Err(CommandRunError::CertificateFileEmpty {
            local_path: path.to_string(),
        }),
        Ok(_) => Err(CommandRunError::CertificateFileNotFound {
            local_path: path.to_string(),
        }),
        Err(_) => Err(CommandRunError::CertificateFileNotFound {
            local_path: path.to_string(),
        }),
    }
}

fn load_ca_certificate(filename: &str) -> Result<Certificate, CommandRunError> {
    validate_certificate_file(filename)?;

    let pem_data = fs::read(filename).map_err(|_| CommandRunError::CertificateFileNotFound {
        local_path: filename.to_string(),
    })?;

    Certificate::from_pem(&pem_data).map_err(|err| {
        CommandRunError::CertificateFileCouldNotBeLoaded1 {
            local_path: filename.to_string(),
            cause: err,
        }
    })
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
    cli::dispatch::dispatch_command_group(
        pair.0,
        pair.1,
        second_level_args,
        client,
        endpoint,
        vhost,
        res_handler,
    )
}

fn dispatch_tanzu_subcommand(
    pair: (&str, &str),
    third_level_args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match &pair {
        ("sds", "status_on_node") => {
            let result =
                tanzu_commands::sds_status_on_node(client, third_level_args).map_err(Into::into);
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
            let result = tanzu_commands::wsr_status(client).map_err(Into::into);
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
        || shared_settings.scheme.is_https()
        || shared_settings.port.unwrap_or(DEFAULT_HTTPS_PORT) == DEFAULT_HTTPS_PORT
}

/// Retrieves a --vhost value, either from global or command-specific arguments
fn virtual_host(shared_settings: &SharedSettings, command_flags: &ArgMatches) -> String {
    // If command defines --vhost and it's not the default, use it
    if let Some(v) = command_flags.try_get_one::<String>("vhost").ok().flatten()
        && v != DEFAULT_VHOST
    {
        return v.clone();
    }
    // Otherwise use the global/shared --vhost flag value
    shared_settings
        .virtual_host
        .clone()
        .unwrap_or_else(|| DEFAULT_VHOST.to_string())
}
