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
mod constants;
mod format;

use crate::cli::SharedFlags;
use crate::constants::DEFAULT_VHOST;
use rabbitmq_http_client::blocking::ClientBuilder;
use rabbitmq_http_client::responses::Overview;
use reqwest::blocking::Client as HTTPClient;

const USER_AGENT: &str = "rabbitmqadmin-ng";
const DEFAULT_TLS_PORT: u16 = 15671;

fn main() {
    let parser = cli::parser();
    let cli = parser.get_matches();

    let sf = SharedFlags::from_args(&cli);
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

    let client = ClientBuilder::new()
        .with_endpoint(endpoint.as_str())
        .with_basic_auth_credentials(sf.username.as_str(), sf.password.as_str())
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

fn should_use_tls(global_flags: &SharedFlags) -> bool {
    global_flags.scheme.to_lowercase() == "https" || global_flags.port == DEFAULT_TLS_PORT
}

/// Retrieves a --vhost value, either from global or command-specific arguments
fn virtual_host(global_flags: &SharedFlags, command_flags: &ArgMatches) -> String {
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
            global_flags.virtual_host.clone()
        }
    } else {
        global_flags.virtual_host.clone()
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
        Ok(_) => {
            println!("Done")
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1)
        }
    }
}
