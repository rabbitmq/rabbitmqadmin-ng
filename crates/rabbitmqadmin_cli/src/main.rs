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
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("list", "vhosts") => {
                    let result = commands::list_vhosts(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                }
                // TODO not implemented yet
                // ("list", "vhost_limits") => {
                //     let result = commands::list_vhost_limits(&cli);
                //     println!("Command execution result:\n\n{:?}", result);
                // }
                ("list", "users") => {
                    let result = commands::list_users(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("list", "connections") => {
                    let result = commands::list_connections(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("list", "channels") => {
                    let result = commands::list_channels(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("list", "consumers") => {
                    let result = commands::list_consumers(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("list", "policies") => {
                    let result = commands::list_policies(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("list", "operator_policies") => {
                    let result = commands::list_operator_policies(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("list", "queues") => {
                    let result = commands::list_queues(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("list", "bindings") => {
                    let result = commands::list_bindings(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("list", "permissions") => {
                    let result = commands::list_permissions(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("list", "parameters") => {
                    let result = commands::list_parameters(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("list", "exchanges") => {
                    let result = commands::list_exchanges(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("declare", "vhost") => {
                    let result = commands::declare_vhost(&cli, command_args);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("delete", "vhost") => {
                    let result = commands::delete_vhost(&cli, command_args);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("delete", "user") => {
                    let result = commands::delete_user(&cli, command_args);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("delete", "queue") => {
                    let result = commands::delete_queue(&cli, command_args);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("purge", "queue") => {
                    let result = commands::purge_queue(&cli, command_args);
                    println!("Command execution result:\n\n{:?}", result);
                }
                _ => {
                    println!("Unknown command and subcommand pair: {:?}", &pair);
                }
            }
        }
    }
}
