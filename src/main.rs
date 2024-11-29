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
use reqwest::Certificate;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process;

use tabled::settings::Style;
use tabled::{Table, Tabled};

use rabbitmq_http_client::blocking::Result;

mod cli;
mod commands;
mod config;
mod constants;
mod format;


use crate::config::SharedSettings;
use crate::constants::{
    DEFAULT_CONFIG_FILE_PATH, DEFAULT_HTTPS_PORT, DEFAULT_NODE_ALIAS, DEFAULT_VHOST,
};
use rabbitmq_http_client::blocking::ClientBuilder;
use rabbitmq_http_client::responses::Overview;
use reqwest::blocking::Client as HTTPClient;

const USER_AGENT: &str = "rabbitmqadmin-ng";

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
        .or(Some(&DEFAULT_NODE_ALIAS.to_string()))
        .cloned();

    let cf_ss = SharedSettings::from_config_file(&config_file_path, node_alias);
    // If the default config file path is used and the function above
    // reports that it is not found, continue. Otherwise exit.
    if cf_ss.is_err() && !uses_default_config_file_path {
        println!(
            "Could not load the provided configuration file at {}",
            config_file_path.to_str().unwrap()
        );
        println!("Underlying error: {}", cf_ss.unwrap_err());
        process::exit(1)
    }
    let sf = if let Ok(val) = cf_ss {
        SharedSettings::from_args_with_defaults(&cli, &val)
    } else {
        SharedSettings::from_args(&cli)
    };
    let endpoint = sf.endpoint();
    let disable_peer_verification = *cli.get_one::<bool>("insecure").unwrap_or(&false);
    let httpc = if should_use_tls(&sf) {
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
            .user_agent(USER_AGENT)
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .danger_accept_invalid_certs(disable_peer_verification)
            .danger_accept_invalid_hostnames(disable_peer_verification);

        if cert.is_some() {
            b = b.add_root_certificate(cert.unwrap());
        }

        b.build()
    } else {
        HTTPClient::builder().user_agent(USER_AGENT).build()
    }
    .unwrap();

    let username = sf.username.clone().unwrap();
    let password = sf.password.clone().unwrap();
    let client = ClientBuilder::new()
        .with_endpoint(endpoint.as_str())
        .with_basic_auth_credentials(username.as_str(), password.as_str())
        .with_client(httpc)
        .build();

    if let Some((verb, group_args)) = cli.subcommand() {
        if let Some((kind, command_args)) = group_args.subcommand() {
            let pair = (verb, kind);

            let vhost = virtual_host(&sf, command_args);

            match &pair {
                ("show", "overview") => {
                    let result = commands::show_overview(client);
                    print_overview_or_fail(result);
                }
                ("show", "churn") => {
                    let result = commands::show_overview(client);
                    print_churn_overview_or_fail(result);
                }
                ("show", "endpoint") => {
                    println!("Using endpoint: {}", endpoint);
                    print_nothing_or_fail(Ok(()))
                }

                ("list", "nodes") => {
                    let result = commands::list_nodes(client);
                    print_table_or_fail(result);
                }
                ("list", "vhosts") => {
                    let result = commands::list_vhosts(client);
                    print_table_or_fail(result);
                }
                ("list", "vhost_limits") => {
                    let result = commands::list_vhost_limits(client, &vhost);
                    print_table_or_fail(result);
                }
                ("list", "user_limits") => {
                    let result = commands::list_user_limits(client, command_args);
                    print_table_or_fail(result);
                }
                ("list", "users") => {
                    let result = commands::list_users(client);
                    print_table_or_fail(result);
                }
                ("list", "connections") => {
                    let result = commands::list_connections(client);
                    print_table_or_fail(result);
                }
                ("list", "channels") => {
                    let result = commands::list_channels(client);
                    print_table_or_fail(result);
                }
                ("list", "consumers") => {
                    let result = commands::list_consumers(client);
                    print_table_or_fail(result);
                }
                ("list", "policies") => {
                    let result = commands::list_policies(client);
                    print_table_or_fail(result);
                }
                ("list", "operator_policies") => {
                    let result = commands::list_operator_policies(client);
                    print_table_or_fail(result);
                }
                ("list", "queues") => {
                    let result = commands::list_queues(client, &vhost);
                    print_table_or_fail(result);
                }
                ("list", "bindings") => {
                    let result = commands::list_bindings(client);
                    print_table_or_fail(result);
                }
                ("list", "permissions") => {
                    let result = commands::list_permissions(client);
                    print_table_or_fail(result);
                }
                ("list", "parameters") => {
                    let result = commands::list_parameters(client, &vhost, command_args);
                    print_table_or_fail(result);
                }
                ("list", "exchanges") => {
                    let result = commands::list_exchanges(client, &vhost);
                    print_table_or_fail(result);
                }
                ("declare", "vhost") => {
                    let result = commands::declare_vhost(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "exchange") => {
                    let result = commands::declare_exchange(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "user") => {
                    let result = commands::declare_user(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "permissions") => {
                    let result = commands::declare_permissions(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "permissions") => {
                    let result = commands::delete_permissions(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "queue") => {
                    let result = commands::declare_queue(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "binding") => {
                    let result = commands::declare_binding(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "policy") => {
                    let result = commands::declare_policy(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "operator_policy") => {
                    let result = commands::declare_operator_policy(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "vhost_limit") => {
                    let result = commands::declare_vhost_limit(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "user_limit") => {
                    let result = commands::declare_user_limit(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "parameter") => {
                    let result = commands::declare_parameter(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "vhost") => {
                    let result = commands::delete_vhost(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "exchange") => {
                    let result = commands::delete_exchange(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "user") => {
                    let result = commands::delete_user(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "queue") => {
                    let result = commands::delete_queue(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "binding") => {
                    let result = commands::delete_binding(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "policy") => {
                    let result = commands::delete_policy(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "operator_policy") => {
                    let result = commands::delete_operator_policy(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "vhost_limit") => {
                    let result = commands::delete_vhost_limit(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "user_limit") => {
                    let result = commands::delete_user_limit(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "parameter") => {
                    let result = commands::delete_parameter(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("purge", "queue") => {
                    let result = commands::purge_queue(client, &vhost, command_args);
                    print_nothing_or_fail(result);
                }
                ("rebalance", "queues") => {
                    let result = commands::rebalance_queues(client);
                    print_nothing_or_fail(result);
                }
                ("close", "connection") => {
                    let result = commands::close_connection(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("definitions", "export") => {
                    let result = commands::export_definitions(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("definitions", "import") => {
                    let result = commands::import_definitions(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("export", "definitions") => {
                    let result = commands::export_definitions(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("import", "definitions") => {
                    let result = commands::import_definitions(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("publish", "message") => {
                    let result = commands::publish_message(client, &vhost, command_args);
                    print_result_or_fail(result);
                }
                ("get", "messages") => {
                    let result = commands::get_messages(client, &vhost, command_args);
                    print_table_or_fail(result);
                }
                _ => {
                    println!("Unknown command and subcommand pair: {:?}", &pair);
                    process::exit(1)
                }
            }
        }
    }
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

fn print_overview_or_fail(result: Result<Overview>) {
    match result {
        Ok(ov) => {
            let mut table = format::overview_table(ov);

            table.with(Style::modern());
            println!("{}", table);
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1)
        }
    }
}

fn print_churn_overview_or_fail(result: Result<Overview>) {
    match result {
        Ok(ov) => {
            let mut table = format::churn_overview_table(ov);

            table.with(Style::modern());
            println!("{}", table);
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1)
        }
    }
}

fn print_table_or_fail<T>(result: Result<Vec<T>>)
where
    T: fmt::Debug + Tabled,
{
    match result {
        Ok(rows) => {
            let mut table = Table::new(rows);
            table.with(Style::modern());
            println!("{}", table);
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1)
        }
    }
}

fn print_result_or_fail<T: fmt::Display>(result: Result<T>) {
    match result {
        Ok(output) => println!("{}", output),
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1)
        }
    }
}

#[allow(dead_code)]
fn print_debug_result_or_fail<T: fmt::Debug>(result: Result<T>) {
    match result {
        Ok(output) => println!("{:?}", output),
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1)
        }
    }
}

fn print_nothing_or_fail<T>(result: Result<T>) {
    match result {
        Ok(_) => (),
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1)
        }
    }
}
