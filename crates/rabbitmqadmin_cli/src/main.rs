use clap::{Arg, Command};

const DEFAULT_HOST: &str = "localhost";
const DEFAULT_PORT: &str = "15672";

fn list_subcommands() -> [Command; 3] {
    [
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
        .arg(Arg::new("node").short('n').required(false))
        .arg(Arg::new("host").short('H').required(false).default_value(DEFAULT_HOST))
        .arg(Arg::new("port").short('P').required(false).value_parser(clap::value_parser!(u16)).default_value(DEFAULT_PORT))
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
    
    println!("cli: {:?}", &cli);
}
