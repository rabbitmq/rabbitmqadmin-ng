mod commands;

use clap::{Arg, Command, ArgMatches};

const DEFAULT_HOST: &str = "localhost";
const DEFAULT_PORT: &str = "15672";
const DEFAULT_PATH_PREFIX: &str = "/api";
const DEFAULT_VHOST: &str = "/";

const DEFAULT_USERNAME: &str = "guest";
const DEFAULT_PASSWORD: &str = "guest";

// default node section in the configuration file
const DEFAULT_NODE_ALIAS: &str = "default";

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

fn main() {
    let cli = Command::new("rabbitmqadmin")
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
        .arg(Arg::new("port").short('P').long("port").required(false).value_parser(clap::value_parser!(u16)).default_value(DEFAULT_PORT))
        // --base-uri
        .arg(Arg::new("base_uri").short('U').long("base-uri").required(false))
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
        .get_matches();
    
    if let Some((verb, group_args)) = cli.subcommand() {
        if let Some((kind, _command_args)) = group_args.subcommand() {
            let pair = (verb, kind);
            println!("{:?}", pair);

            let _cmd = match pair {
                ("list", "nodes") => list_nodes_command(&cli),
                ("list", "") => list_users_command(&cli),
                _ => ()
            };

            // cmd.execute();
        }
    }
}

struct SharedFlags {
    pub hostname: String,
    pub port: u32,
    pub base_uri: String,
    pub path_prefix: String,

    pub username: String,
    pub password: String,

    pub virtual_host: String
}

fn list_nodes_command(general_args: &ArgMatches) {
    let target_hostname = general_args.get_one::<String>("host").unwrap();
    println!("--host: {}", target_hostname);
}

fn list_users_command(general_args: &ArgMatches) {
    let target_hostname = general_args.get_one::<String>("host").unwrap();
    println!("--host: {}", target_hostname);
}