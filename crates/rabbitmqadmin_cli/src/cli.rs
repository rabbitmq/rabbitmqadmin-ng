use super::constants::*;
use clap::{Arg, Command};

pub fn parser() -> Command {
    Command::new("rabbitmqadmin")
        .version("0.1.0")
        .author("Michael Klishin")
        .about("rabbitmqadmin v2")
        .disable_version_flag(true)
        // --node
        // This is NOT the same as --node in case of rabbitmqctl, rabbitmq-diagnostics, etc.
        // This is node section name in the configuration file. MK.
        .arg(Arg::new("node").short('N').long("node").required(false).default_value(DEFAULT_NODE_ALIAS))
        // --host
        .arg(Arg::new("host").short('H').long("host").required(false).default_value(DEFAULT_HOST)).visible_alias("hostname")
        // --port
        .arg(Arg::new("port").short('P').long("port").required(false).value_parser(clap::value_parser!(u16)).default_value(DEFAULT_PORT_STR))
        // --base-uri
        .arg(Arg::new("base_uri").short('U').long("base-uri").required(false).conflicts_with_all(["host", "port"]))
        // --path-prefix
        .arg(Arg::new("path_prefix").long("path-prefix").required(false).default_value(DEFAULT_PATH_PREFIX))
        // --vhost
        .arg(Arg::new("vhost").short('V').long("vhost").required(false).default_value(DEFAULT_VHOST))
        // --username
        .arg(Arg::new("username").short('u').long("username").required(false).default_value(DEFAULT_USERNAME))
        // --password
        .arg(Arg::new("password").short('p').long("password").required(false).default_value(DEFAULT_PASSWORD).requires("username"))
        // --quiet
        .arg(Arg::new("quiet").short('q').long("quiet").required(false)
                .value_parser(clap::value_parser!(bool)).action(clap::ArgAction::SetTrue))
        .subcommand_required(true)
        .subcommand_value_name("command")
        .subcommands([
            Command::new("show")
                .about("overview")
                .subcommand_value_name("summary")
                .subcommands(show_subcomands()),
            Command::new("list")
                .about("lists objects by type")
                .subcommand_value_name("objects")
                .subcommands(list_subcommands()),
            Command::new("declare")
                .about("creates or declares things")
                .subcommand_value_name("object")
                .subcommands(declare_subcommands()),
            Command::new("delete")
                .about("deletes objects"),
            Command::new("purge")
                .about("deletes data")
        ])
}

fn list_subcommands() -> [Command; 4] {
    [
        Command::new("nodes"),
        Command::new("users"),
        Command::new("vhosts"),
        Command::new("permissions")
    ]
}

fn declare_subcommands() -> [Command; 9] {
    [
        Command::new("user")
            .about("creates a user")
            .arg(Arg::new("name").help("username"))
            .arg(Arg::new("password_hash").long_help("salted password hash, see https://rabbitmq.com/passwords.html"))
            .arg(Arg::new("password").long_help("prefer providing a hash, see https://rabbitmq.com/passwords.html"))
            .arg(Arg::new("tags").long_help("a list of comma-separated tags")),
        Command::new("vhost")
            .about("creates a virtual host"),
        Command::new("permission").about("grants a permission"),
        Command::new("queue"),
        Command::new("exchange"),
        Command::new("binding"),
        Command::new("parameter").about("sets a runtime parameter"),
        Command::new("policy").about("creates or updates a policy"),
        Command::new("operator_policy").about("creates or updates an operator policy"),
        ]
}

fn show_subcomands() -> [Command; 1] {
    [
        Command::new("overview")
            .about("displays a subset of aggregated metrics found on the Overview page in management UI")
    ]
}