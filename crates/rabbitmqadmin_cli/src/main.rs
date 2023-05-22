mod cli;
mod commands;
mod constants;

fn main() {
    let parser = cli::parser();
    let cli = parser.get_matches();

    if let Some((verb, group_args)) = cli.subcommand() {
        if let Some((kind, _command_args)) = group_args.subcommand() {
            let pair = (verb, kind);

            match &pair {
                ("list", "nodes") => {
                    let result = commands::list_nodes(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                },
                ("list", "vhosts") => {
                    let result = commands::list_vhosts(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                },
                ("list", "users") => {
                    let result = commands::list_users(&cli);
                    println!("Command execution result:\n\n{:?}", result);
                }
                _ => {
                    println!("Unknown command and subcommand pair: {:?}", &pair);
                }
            }
        }
    }
}
