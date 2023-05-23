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
                ("delete", "vhost") => {
                    let result = commands::delete_vhost(&cli, &command_args);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("delete", "user") => {
                    let result = commands::delete_user(&cli, &command_args);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("delete", "queue") => {
                    let result = commands::delete_queue(&cli, &command_args);
                    println!("Command execution result:\n\n{:?}", result);
                }
                ("purge", "queue") => {
                    let result = commands::purge_queue(&cli, &command_args);
                    println!("Command execution result:\n\n{:?}", result);
                }
                _ => {
                    println!("Unknown command and subcommand pair: {:?}", &pair);
                }
            }
        }
    }
}
