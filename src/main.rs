use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::{error::Error, process};

use tabled::settings::Style;
use tabled::{Table, Tabled};

mod cli;
mod commands;
mod constants;

use crate::cli::SharedFlags;
use rabbitmq_http_client::blocking::Client as APIClient;

fn main() {
    let parser = cli::parser();
    let cli = parser.get_matches();

    let sf = SharedFlags::from_args(&cli);
    let endpoint = sf.endpoint();
    let mut client =
        APIClient::new(&endpoint).with_basic_auth_credentials(&sf.username, &sf.password);

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

        match client.with_pem_ca_certificate(pem) {
            Ok(c) => client = c,
            Err(err) => {
                eprintln!(
                    "{} doesn't seem to be a valid PEM file: {}",
                    pem_file.to_string_lossy(),
                    err
                );
                process::exit(1);
            }
        }
    }

    if *cli.get_one::<bool>("insecure").unwrap_or(&false) {
        client = client.without_tls_peer_verification();
    }

    if let Some((verb, group_args)) = cli.subcommand() {
        if let Some((kind, command_args)) = group_args.subcommand() {
            let pair = (verb, kind);

            match &pair {
                ("list", "nodes") => {
                    let result = commands::list_nodes(client);
                    print_table_or_fail(result);
                }
                ("list", "vhosts") => {
                    let result = commands::list_vhosts(client);
                    print_table_or_fail(result);
                }
                ("list", "vhost_limits") => {
                    let result = commands::list_vhost_limits(client, &sf.virtual_host);
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
                    let result = commands::list_queues(client, &sf.virtual_host);
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
                    let result = commands::list_parameters(client, &sf.virtual_host, command_args);
                    print_table_or_fail(result);
                }
                ("list", "exchanges") => {
                    let result = commands::list_exchanges(client, &sf.virtual_host);
                    print_table_or_fail(result);
                }
                ("declare", "vhost") => {
                    let result = commands::declare_vhost(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "exchange") => {
                    let result = commands::declare_exchange(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "user") => {
                    let result = commands::declare_user(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "permissions") => {
                    let result =
                        commands::declare_permissions(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "permissions") => {
                    let result =
                        commands::delete_permissions(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "queue") => {
                    let result = commands::declare_queue(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "binding") => {
                    let result = commands::declare_binding(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "policy") => {
                    let result = commands::declare_policy(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "operator_policy") => {
                    let result =
                        commands::declare_operator_policy(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "vhost_limit") => {
                    let result =
                        commands::declare_vhost_limit(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "user_limit") => {
                    let result = commands::declare_user_limit(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "parameter") => {
                    let result =
                        commands::declare_parameter(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "vhost") => {
                    let result = commands::delete_vhost(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "exchange") => {
                    let result = commands::delete_exchange(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "user") => {
                    let result = commands::delete_user(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "queue") => {
                    let result = commands::delete_queue(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "binding") => {
                    let result = commands::delete_binding(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "policy") => {
                    let result = commands::delete_policy(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "operator_policy") => {
                    let result =
                        commands::delete_operator_policy(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "vhost_limit") => {
                    let result =
                        commands::delete_vhost_limit(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "user_limit") => {
                    let result = commands::delete_user_limit(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "parameter") => {
                    let result = commands::delete_parameter(client, &sf.virtual_host, command_args);
                    print_nothing_or_fail(result);
                }
                ("purge", "queue") => {
                    let result = commands::purge_queue(client, &sf.virtual_host, command_args);
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
                ("export", "definitions") => {
                    let result = commands::export_definitions(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("import", "definitions") => {
                    let result = commands::import_definitions(client, command_args);
                    print_nothing_or_fail(result);
                }
                ("publish", "message") => {
                    let result = commands::publish_message(client, &sf.virtual_host, command_args);
                    print_result_or_fail(result);
                }
                ("get", "messages") => {
                    let result = commands::get_messages(client, &sf.virtual_host, command_args);
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

fn print_table_or_fail<T>(result: Result<Vec<T>, rabbitmq_http_client::blocking::Error>)
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
            eprintln!("{}", error.source().unwrap_or(&error),);
            process::exit(1)
        }
    }
}

fn print_result_or_fail<T: fmt::Display>(result: Result<T, rabbitmq_http_client::blocking::Error>) {
    match result {
        Ok(output) => println!("{}", output),
        Err(error) => {
            eprintln!("{}", error.source().unwrap_or(&error),);
            process::exit(1)
        }
    }
}

#[allow(dead_code)]
fn print_debug_result_or_fail<T: fmt::Debug>(
    result: Result<T, rabbitmq_http_client::blocking::Error>,
) {
    match result {
        Ok(output) => println!("{:?}", output),
        Err(error) => {
            eprintln!("{}", error.source().unwrap_or(&error),);
            process::exit(1)
        }
    }
}

fn print_nothing_or_fail<T>(result: Result<T, rabbitmq_http_client::blocking::Error>) {
    match result {
        Ok(_) => {
            println!("Done")
        }
        Err(error) => {
            eprintln!("{}", error.source().unwrap_or(&error),);
            process::exit(1)
        }
    }
}
