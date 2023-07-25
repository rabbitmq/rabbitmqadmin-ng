use std::fmt;
use std::{error::Error, process};

use tabled::{Table, Tabled};

mod cli;
mod commands;
mod constants;

fn main() {
    let parser = cli::parser();
    let cli = parser.get_matches();

    if let Some((verb, group_args)) = cli.subcommand() {
        if let Some((kind, command_args)) = group_args.subcommand() {
            let pair = (verb, kind);

            match &pair {
                ("list", "nodes") => {
                    let result = commands::list_nodes(&cli);
                    print_table_or_fail(result);
                }
                ("list", "vhosts") => {
                    let result = commands::list_vhosts(&cli);
                    print_result_or_fail(result);
                }
                ("list", "vhost_limits") => {
                    let result = commands::list_vhost_limits(&cli);
                    print_result_or_fail(result);
                }
                ("list", "user_limits") => {
                    let result = commands::list_user_limits(&cli, command_args);
                    print_result_or_fail(result);
                }
                ("list", "users") => {
                    let result = commands::list_users(&cli);
                    print_result_or_fail(result);
                }
                ("list", "connections") => {
                    let result = commands::list_connections(&cli);
                    print_result_or_fail(result);
                }
                ("list", "channels") => {
                    let result = commands::list_channels(&cli);
                    print_result_or_fail(result);
                }
                ("list", "consumers") => {
                    let result = commands::list_consumers(&cli);
                    print_result_or_fail(result);
                }
                ("list", "policies") => {
                    let result = commands::list_policies(&cli);
                    print_result_or_fail(result);
                }
                ("list", "operator_policies") => {
                    let result = commands::list_operator_policies(&cli);
                    print_result_or_fail(result);
                }
                ("list", "queues") => {
                    let result = commands::list_queues(&cli);
                    print_result_or_fail(result);
                }
                ("list", "bindings") => {
                    let result = commands::list_bindings(&cli);
                    print_result_or_fail(result);
                }
                ("list", "permissions") => {
                    let result = commands::list_permissions(&cli);
                    print_result_or_fail(result);
                }
                ("list", "parameters") => {
                    let result = commands::list_parameters(&cli, command_args);
                    print_result_or_fail(result);
                }
                ("list", "exchanges") => {
                    let result = commands::list_exchanges(&cli);
                    print_result_or_fail(result);
                }
                ("declare", "vhost") => {
                    let result = commands::declare_vhost(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "exchange") => {
                    let result = commands::declare_exchange(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "user") => {
                    let result = commands::declare_user(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "permissions") => {
                    let result = commands::declare_permissions(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "permissions") => {
                    let result = commands::delete_permissions(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "queue") => {
                    let result = commands::declare_queue(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "binding") => {
                    let result = commands::declare_binding(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "policy") => {
                    let result = commands::declare_policy(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "operator_policy") => {
                    let result = commands::declare_operator_policy(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "vhost_limit") => {
                    let result = commands::declare_vhost_limit(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "user_limit") => {
                    let result = commands::declare_user_limit(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("declare", "parameter") => {
                    let result = commands::declare_parameter(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "vhost") => {
                    let result = commands::delete_vhost(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "exchange") => {
                    let result = commands::delete_exchange(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "user") => {
                    let result = commands::delete_user(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "queue") => {
                    let result = commands::delete_queue(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "binding") => {
                    let result = commands::delete_binding(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "policy") => {
                    let result = commands::delete_policy(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "operator_policy") => {
                    let result = commands::delete_operator_policy(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "vhost_limit") => {
                    let result = commands::delete_vhost_limit(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "user_limit") => {
                    let result = commands::delete_user_limit(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("delete", "parameter") => {
                    let result = commands::delete_parameter(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("purge", "queue") => {
                    let result = commands::purge_queue(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                ("close", "connection") => {
                    let result = commands::close_connection(&cli, command_args);
                    print_nothing_or_fail(result);
                }
                _ => {
                    println!("Unknown command and subcommand pair: {:?}", &pair);
                }
            }
        }
    }
}

fn print_table_or_fail<T>(result: Result<Vec<T>, rabbitmq_http_client::blocking::Error>)
where T: fmt::Debug + Tabled {
    match result {
        Ok(rows) => {
            println!("Rows: {:?}", rows);
            let table = Table::new(rows);
            println!("Table: {}", table.to_string());
        },
        Err(error) => {
            eprintln!("{}", error.source().unwrap_or(&error),);
            process::exit(1)
        }
    }
}

fn print_result_or_fail<T: fmt::Debug>(result: Result<Vec<T>, rabbitmq_http_client::blocking::Error>) {
    match result {
        Ok(rows) => {
            println!("Rows: {:?}", rows)
        },
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
        },
        Err(error) => {
            eprintln!("{}", error.source().unwrap_or(&error),);
            process::exit(1)
        }
    }
}