use super::constants::*;
use clap::{Arg, ArgAction, ArgMatches, Command};
use rabbitmq_http_client::commons::QueueType;
use url::Url;

#[derive(Debug, Clone)]
pub struct SharedFlags {
    pub hostname: String,
    pub port: u16,
    pub path_prefix: Option<String>,

    pub username: String,
    pub password: String,

    pub virtual_host: String,
}

impl SharedFlags {
    pub fn from_args(general_args: &ArgMatches) -> SharedFlags {
        if let Some(base_uri) = general_args.get_one::<String>("base_uri") {
            let url = Url::parse(base_uri).unwrap();
            SharedFlags::new_from_uri(&url, general_args)
        } else {
            SharedFlags::new(general_args)
        }
    }

    pub fn new(cli_args: &ArgMatches) -> Self {
        let default_hostname = DEFAULT_HOST.to_owned();
        let hostname = cli_args
            .get_one::<String>("host")
            .unwrap_or(&default_hostname);
        let port = cli_args.get_one::<u16>("port").unwrap();
        let path_prefix = cli_args.get_one::<String>("path_prefix").cloned();
        let username = cli_args.get_one::<String>("username").unwrap();
        let password = cli_args.get_one::<String>("password").unwrap();
        let default_vhost = DEFAULT_VHOST.to_owned();
        let vhost = cli_args
            .get_one::<String>("vhost")
            .unwrap_or(&default_vhost);

        Self {
            hostname: hostname.clone(),
            port: (*port),
            path_prefix,
            username: username.clone(),
            password: password.clone(),
            virtual_host: vhost.clone(),
        }
    }

    pub fn new_from_uri(url: &Url, cli_args: &ArgMatches) -> Self {
        let hostname = url.host_str().unwrap_or(DEFAULT_HOST).to_string();
        let port = url.port().unwrap_or(DEFAULT_HTTP_PORT);
        let path_prefix = cli_args.get_one::<String>("path_prefix").cloned();
        let username = cli_args.get_one::<String>("username").unwrap();
        let password = cli_args.get_one::<String>("password").unwrap();
        let default_vhost = DEFAULT_VHOST.to_owned();
        let vhost = cli_args
            .get_one::<String>("vhost")
            .unwrap_or(&default_vhost);

        Self {
            hostname,
            port,
            path_prefix,
            username: username.clone(),
            password: password.clone(),
            virtual_host: vhost.clone(),
        }
    }

    pub fn endpoint(&self) -> String {
        match &self.path_prefix {
            Some(prefix) => format!("http://{}:{}{}/api", self.hostname, self.port, prefix),
            None => format!("http://{}:{}/api", self.hostname, self.port),
        }
    }
}

pub fn parser() -> Command {
    Command::new("rabbitmqadmin")
        .version("0.1.0")
        .author("Michael Klishin")
        .about("rabbitmqadmin v2")
        .disable_version_flag(true)
        // --node
        // This is NOT the same as --node in case of rabbitmqctl, rabbitmq-diagnostics, etc.
        // This is node section name in the configuration file. MK.
        .arg(
            Arg::new("node")
                .short('N')
                .long("node")
                .required(false)
                .default_value(DEFAULT_NODE_ALIAS),
        )
        // --host
        .arg(
            Arg::new("host")
                .short('H')
                .long("host")
                .required(false)
                .default_value(DEFAULT_HOST),
        )
        .visible_alias("hostname")
        // --port
        .arg(
            Arg::new("port")
                .short('P')
                .long("port")
                .required(false)
                .value_parser(clap::value_parser!(u16))
                .default_value(DEFAULT_PORT_STR),
        )
        // --base-uri
        .arg(
            Arg::new("base_uri")
                .short('U')
                .long("base-uri")
                .required(false)
                .conflicts_with_all(["host", "port"]),
        )
        // --path-prefix
        .arg(
            Arg::new("path_prefix")
                .long("path-prefix")
                .required(false)
                .default_value(DEFAULT_PATH_PREFIX),
        )
        // --vhost
        .arg(
            Arg::new("vhost")
                .short('V')
                .long("vhost")
                .required(false)
                .default_value(DEFAULT_VHOST),
        )
        // --username
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .required(false)
                .default_value(DEFAULT_USERNAME),
        )
        // --password
        .arg(
            Arg::new("password")
                .short('p')
                .long("password")
                .required(false)
                .default_value(DEFAULT_PASSWORD)
                .requires("username"),
        )
        // --quiet
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .required(false)
                .value_parser(clap::value_parser!(bool))
                .action(clap::ArgAction::SetTrue),
        )
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
                .about("deletes objects")
                .subcommand_value_name("object")
                .subcommands(delete_subcommands()),
            Command::new("purge")
                .about("purges queues")
                .subcommand_value_name("queue")
                .subcommands(purge_subcommands()),
        ])
}

fn list_subcommands() -> [Command; 14] {
    [
        Command::new("nodes"),
        Command::new("users"),
        Command::new("vhosts"),
        Command::new("permissions"),
        Command::new("connections"),
        Command::new("channels"),
        Command::new("queues"),
        Command::new("exchanges"),
        Command::new("bindings"),
        Command::new("consumers"),
        Command::new("parameters"),
        Command::new("policies"),
        Command::new("operator_policies"),
        Command::new("vhost_limits"),
    ]
}

fn declare_subcommands() -> [Command; 9] {
    [
        Command::new("user")
            .about("creates a user")
            .arg(Arg::new("name").long("name").help("username"))
            .arg(
                Arg::new("password_hash")
                    .long_help("salted password hash, see https://rabbitmq.com/passwords.html"),
            )
            .arg(
                Arg::new("password")
                    .long_help("prefer providing a hash, see https://rabbitmq.com/passwords.html"),
            )
            .arg(Arg::new("tags").long_help("a list of comma-separated tags")),
        Command::new("vhost")
            .about("creates a virtual host")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("virtual host name")
                    .required(true),
            )
            .arg(
                Arg::new("default_queue_type")
                    .long("default-queue-type")
                    .required(false)
                    .default_value(DEFAULT_QUEUE_TYPE)
                    .help("default queue type, one of: classic, quorum, stream"),
            )
            .arg(
                Arg::new("description")
                    .long("description")
                    .required(false)
                    .help("a brief description of this virtual host"),
            )
            .arg(
                Arg::new("tracing")
                    .long("tracing")
                    .required(false)
                    .action(ArgAction::SetTrue)
                    .help("should tracing be enabled for this virtual host?"),
            ),
        Command::new("permission").about("grants a permission"),
        Command::new("queue")
            .about("declares a queue")
            .arg(Arg::new("name").long("name").required(true).help("name"))
            .arg(
                Arg::new("type")
                    .long("type")
                    .help("queue type")
                    .value_parser(clap::value_parser!(QueueType))
                    .required(true),
            )
            .arg(
                Arg::new("durable")
                    .long("durable")
                    .help("should it persist after a restart")
                    .required(false)
                    .value_parser(clap::value_parser!(bool)),
            )
            .arg(
                Arg::new("auto_delete")
                    .long("auto_delete")
                    .help("should it be deleted when the last consumer disconnects")
                    .required(false)
                    .value_parser(clap::value_parser!(bool)),
            )
            .arg(
                Arg::new("arguments")
                    .long("arguments")
                    .help("additional exchange arguments")
                    .required(false)
                    .default_value("{}")
                    .value_parser(clap::value_parser!(String)),
            ),
        Command::new("exchange")
            .about("declares an exchange")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("exchange name")
                    .required(true),
            )
            .arg(
                Arg::new("type")
                    .long("type")
                    .help("exchange type")
                    .required(false),
            )
            .arg(
                Arg::new("durable")
                    .long("durable")
                    .help("should it persist after a restart")
                    .required(false)
                    .value_parser(clap::value_parser!(bool)),
            )
            .arg(
                Arg::new("auto_delete")
                    .long("auto_delete")
                    .help("should it be deleted when the last queue is unbound")
                    .required(false)
                    .value_parser(clap::value_parser!(bool)),
            )
            .arg(
                Arg::new("arguments")
                    .long("arguments")
                    .help("additional exchange arguments")
                    .required(false)
                    .default_value("{}")
                    .value_parser(clap::value_parser!(String)),
            ),
        Command::new("binding").about("binds to an exchange"),
        Command::new("parameter").about("sets a runtime parameter"),
        Command::new("policy").about("creates or updates a policy"),
        Command::new("operator_policy").about("creates or updates an operator policy"),
    ]
}

fn show_subcomands() -> [Command; 1] {
    [Command::new("overview").about(
        "displays a subset of aggregated metrics found on the Overview page in management UI",
    )]
}

fn delete_subcommands() -> [Command; 9] {
    [
        Command::new("user").about("deletes a user").arg(
            Arg::new("name")
                .long("name")
                .help("username")
                .required(true),
        ),
        Command::new("vhost").about("deletes a virtual host").arg(
            Arg::new("name")
                .long("name")
                .help("virtual host")
                .required(true),
        ),
        Command::new("permission").about("revokes a permission"),
        Command::new("queue")
            .about("deletes a queue")
            .arg(Arg::new("name").long("name").help("queue").required(true)),
        Command::new("exchange").about("deletes an exchange").arg(
            Arg::new("name")
                .long("name")
                .help("exchange")
                .required(true),
        ),
        Command::new("binding").about("deletes a binding"),
        Command::new("parameter").about("clears a runtime parameter"),
        Command::new("policy").about("deletes a policy"),
        Command::new("operator_policy").about("deletes an operator policy"),
    ]
}

fn purge_subcommands() -> [Command; 1] {
    [Command::new("queue").about("purges (empties) a queue").arg(
        Arg::new("name")
            .long("name")
            .help("queue name")
            .required(true),
    )]
}