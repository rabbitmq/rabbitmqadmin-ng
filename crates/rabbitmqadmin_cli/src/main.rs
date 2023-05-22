mod cli;
mod commands;
mod constants;

use clap::ArgMatches;
use cli::SharedFlags;

use rabbitmq_http_client::Client as APIClient;
use rabbitmq_http_client::responses::Result as ClientResult;
use rabbitmq_http_client::responses;

fn main() {
    let parser = cli::parser();
    let cli = parser.get_matches();

    if let Some((verb, group_args)) = cli.subcommand() {
        if let Some((kind, _command_args)) = group_args.subcommand() {
            let pair = (verb, kind);

            if ("list", "nodes") == pair {
                let result = run_list_nodes_command(&cli);
                println!("Command execution result:\n\n{:?}", result);
            };

            if ("list", "vhosts") == pair {
                let result = run_list_vhosts_command(&cli);
                println!("Command execution result:\n\n{:?}", result);
            };

            if ("list", "users") == pair {
                let result = run_list_users_command(&cli);
                println!("Command execution result:\n\n{:?}", result);
            };
        }
    }
}

fn run_list_nodes_command(general_args: &ArgMatches) -> ClientResult<Vec<responses::ClusterNode>> {
    let sf = SharedFlags::from_args(general_args);

    let endpoint = sf.endpoint();
    let rc =
        APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, Some(&sf.password));
    rc.list_nodes()
}

fn run_list_vhosts_command(general_args: &ArgMatches) -> ClientResult<Vec<responses::VirtualHost>> {
    let sf = SharedFlags::from_args(general_args);

    let endpoint = sf.endpoint();
    let rc =
        APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, Some(&sf.password));
    rc.list_vhosts()
}

fn run_list_users_command(general_args: &ArgMatches) -> ClientResult<Vec<responses::User>> {
    let sf = SharedFlags::from_args(general_args);

    let endpoint = sf.endpoint();
    let rc =
        APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, Some(&sf.password));
    rc.list_users()
}
