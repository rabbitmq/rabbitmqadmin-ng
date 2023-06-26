use std::fmt;
use std::{error::Error, process};

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
                    print_result_or_fail(result);
                }
                ("list", "vhosts") => {
                    let result = commands::list_vhosts(&cli);
                    print_result_or_fail(result);
                }
                ("list", "vhost_limits") => {
                    let result = commands::list_vhost_limits(&cli);
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
                    let result = commands::list_parameters(&cli);
                    print_result_or_fail(result);
                }
                ("list", "exchanges") => {
                    let result = commands::list_exchanges(&cli);
                    print_result_or_fail(result);
                }
                ("declare", "vhost") => {
                    let result = commands::declare_vhost(&cli, command_args);
                    print_result_or_fail(result);
                }
                ("declare", "exchange") => {
                    let result = commands::declare_exchange(&cli, command_args);
                    print_result_or_fail(result);
                }
                ("declare", "user") => {
                    let result = commands::declare_user(&cli, command_args);
                    print_result_or_fail(result);
                }
                ("declare", "queue") => {
                    let result = commands::declare_queue(&cli, command_args);
                    print_result_or_fail(result);
                }
                ("delete", "vhost") => {
                    let result = commands::delete_vhost(&cli, command_args);
                    print_result_or_fail(result);
                }
                ("delete", "exchange") => {
                    let result = commands::delete_exchange(&cli, command_args);
                    print_result_or_fail(result);
                }
                ("delete", "user") => {
                    let result = commands::delete_user(&cli, command_args);
                    print_result_or_fail(result);
                }
                ("delete", "queue") => {
                    let result = commands::delete_queue(&cli, command_args);
                    print_result_or_fail(result);
                }
                ("purge", "queue") => {
                    let result = commands::purge_queue(&cli, command_args);
                    print_result_or_fail(result);
                }
                _ => {
                    println!("Unknown command and subcommand pair: {:?}", &pair);
                }
            }
        }
    }
}

fn print_result_or_fail<T: fmt::Debug>(result: Result<T, rabbitmq_http_client::blocking::Error>) {
    match result {
        Ok(output) => println!("{:?}", output),
        Err(error) => {
            eprintln!("{}", error.source().unwrap_or(&error),);
            process::exit(1)
        }
    }
}
