mod constants;
mod cli;
mod commands;

use constants::*;
use clap::{ArgMatches};
use url::Url;

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

#[derive(Debug, Clone)]
pub struct SharedFlags {
    pub hostname: String,
    pub port: u16,
    pub path_prefix: String,

    pub username: String,
    pub password: String,

    pub virtual_host: String
}

impl SharedFlags {
    pub fn new(cli_args: &ArgMatches) -> Self {
        let default_hostname = DEFAULT_HOST.to_owned();
        let hostname = cli_args.get_one::<String>("host").unwrap_or(&default_hostname);
        let port = cli_args.get_one::<u16>("port").unwrap();
        let default_path_prefix = DEFAULT_PATH_PREFIX.to_owned();
        let path_prefix = cli_args.get_one::<String>("path_prefix").unwrap_or(&default_path_prefix);
        let username = cli_args.get_one::<String>("username").unwrap();
        let password = cli_args.get_one::<String>("password").unwrap();
        let default_vhost = DEFAULT_VHOST.to_owned();
        let vhost = cli_args.get_one::<String>("vhost").unwrap_or(&default_vhost);

        Self {
            hostname: hostname.clone(),
            port: (*port).clone(),
            path_prefix: path_prefix.clone(),
            username: username.clone(),
            password: password.clone(),
            virtual_host: vhost.clone()
        }
    }

    pub fn new_from_uri(url: &Url, cli_args: &ArgMatches) -> Self {
        let hostname = url.host_str().unwrap_or(&DEFAULT_HOST).to_owned();
        let port = url.port().unwrap_or(DEFAULT_HTTP_PORT);
        let default_path_prefix = DEFAULT_PATH_PREFIX.to_owned();
        let path_prefix = cli_args.get_one::<String>("path_prefix").unwrap_or(&default_path_prefix);
        let username = cli_args.get_one::<String>("username").unwrap();
        let password = cli_args.get_one::<String>("password").unwrap();
        let default_vhost = DEFAULT_VHOST.to_owned();
        let vhost = cli_args.get_one::<String>("vhost").unwrap_or(&default_vhost);

        Self {
            hostname: hostname.clone(),
            port: port.clone(),
            path_prefix: path_prefix.clone(),
            username: username.clone(),
            password: password.clone(),
            virtual_host: vhost.clone()
        }
    }
}

fn shared_flags_from(general_args: &ArgMatches) -> SharedFlags {
    if let Some(base_uri) = general_args.get_one::<String>("base_uri") {
        let url = Url::parse(&base_uri).unwrap();
        SharedFlags::new_from_uri(&url, &general_args)
    } else {
        SharedFlags::new(general_args)
    }
}

fn list_nodes_command(general_args: &ArgMatches) {
    let shared: SharedFlags = shared_flags_from(general_args);
    println!("list_nodes shared: {:?}", shared);


}

fn list_users_command(general_args: &ArgMatches) {
    let shared: SharedFlags = shared_flags_from(general_args);
    println!("list_users shared: {:?}", shared)
}