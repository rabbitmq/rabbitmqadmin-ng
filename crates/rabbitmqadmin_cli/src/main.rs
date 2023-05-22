mod constants;
mod cli;
mod commands;

use cli::SharedFlags;
use clap::{ArgMatches};

fn main() {
    let parser = cli::parser();
    let cli = parser.get_matches();
    
    if let Some((verb, group_args)) = cli.subcommand() {
        if let Some((kind, _command_args)) = group_args.subcommand() {
            let pair = (verb, kind);
            println!("{:?}", pair);

            let _cmd = match pair {
                ("list", "nodes") => list_nodes_command(&cli),
                ("list", "users") => list_users_command(&cli),
                _ => ()
            };

            // cmd.execute();
        }
    }
}

fn list_nodes_command(general_args: &ArgMatches) {
    let shared = SharedFlags::from(general_args);
    println!("list_nodes shared: {:?}", shared);


}

fn list_users_command(general_args: &ArgMatches) {
    let shared = SharedFlags::from(general_args);
    println!("list_users shared: {:?}", shared)
}