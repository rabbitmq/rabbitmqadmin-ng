use std::path::PathBuf;

use super::constants::*;
use clap::{Arg, ArgAction, ArgMatches, Command};
use rabbitmq_http_client::commons::{BindingDestinationType, QueueType};
use url::Url;

#[derive(Debug, Clone)]
pub struct SharedFlags {
    pub scheme: String,
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
            scheme: "http".to_string(),
            hostname: hostname.clone(),
            port: (*port),
            path_prefix,
            username: username.clone(),
            password: password.clone(),
            virtual_host: vhost.clone(),
        }
    }

    pub fn new_from_uri(url: &Url, cli_args: &ArgMatches) -> Self {
        let scheme = url.scheme().to_string();
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
            scheme,
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
            Some(prefix) => format!(
                "{}://{}:{}{}/api",
                self.scheme, self.hostname, self.port, prefix
            ),
            None => format!("{}://{}:{}/api", self.scheme, self.hostname, self.port),
        }
    }
}

pub fn parser() -> Command {
    let after_help: &'static str = color_print::cstr!(
        r#"
<bold><yellow>Getting Help</yellow></bold>
  RabbitMQ docs: https://rabbitmq.com/docs/
  Discord server: https://rabbitmq.com/discord/
  GitHub Discussions: https://github.com/rabbitmq/rabbitmq-server/discussions"#
    );

    Command::new("rabbitmqadmin")
        .version(clap::crate_version!())
        .author("RabbitMQ Core Team")
        .about("rabbitmqadmin gen 2")
        .long_about("RabbitMQ CLI that uses the HTTP API")
        .after_help(after_help)
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
                .help("HTTP API hostname to use when connecting")
                .required(false)
                .default_value(DEFAULT_HOST),
        )
        .visible_alias("hostname")
        // --port
        .arg(
            Arg::new("port")
                .short('P')
                .long("port")
                .help("HTTP API port to use when connecting")
                .required(false)
                .value_parser(clap::value_parser!(u16))
                .default_value(DEFAULT_PORT_STR),
        )
        // --base-uri
        .arg(
            Arg::new("base_uri")
                .short('U')
                .long("base-uri")
                .help("base HTTP API endpoint URI")
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
                .help("target virtual host")
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
        // --insecure
        .arg(
            Arg::new("insecure")
                .short('k')
                .long("insecure")
                .required(false)
                .help("disables TLS peer (certificate chain) verification")
                .value_parser(clap::value_parser!(bool))
                .action(clap::ArgAction::SetTrue),
        )
        // --tls-ca-cert-file
        .arg(
            Arg::new("tls-ca-cert-file")
                .long("tls-ca-cert-file")
                .required(false)
                .help("TLS CA certificate PEM file path")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        // --quiet
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("produce less output")
                .required(false)
                .value_parser(clap::value_parser!(bool))
                .action(clap::ArgAction::SetTrue),
        )
        .subcommand_required(true)
        .subcommand_value_name("command")
        .subcommands([
            Command::new("show")
                .about("overview")
                .after_long_help(color_print::cstr!(
                    "<bold>Doc guide</bold>: https://rabbitmq.com/monitoring.html"
                ))
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
            Command::new("close")
                .about("closes connections")
                .subcommand_value_name("connection")
                .subcommands(close_subcommands()),
            Command::new("rebalance")
                .about("rebalances queue leaders")
                .after_long_help(color_print::cstr!(
                    "<bold>Doc guide</bold>: https://www.rabbitmq.com/docs/quorum-queues"
                ))
                .subcommand_value_name("queues")
                .subcommands(rebalance_subcommands()),
            Command::new("definitions")
                .about("operations on definitions")
                .after_long_help(color_print::cstr!(
                    "<bold>Doc guide</bold>: https://rabbitmq.com/docs/definitions"
                ))
                .subcommand_value_name("export")
                .subcommands(definitions_subcommands()),
            Command::new("export")
                .about("see 'definitions export'")
                .after_long_help(color_print::cstr!(
                    "<bold>Doc guide</bold>: https://rabbitmq.com/docs/definitions"
                ))
                .subcommand_value_name("definitions")
                .subcommands(export_subcommands()),
            Command::new("import")
                .about("see 'definitions import'")
                .after_long_help(color_print::cstr!(
                    "<bold>Doc guide</bold>: https://rabbitmq.com/docs/definitions"
                ))
                .subcommand_value_name("definitions")
                .subcommands(import_subcommands()),
            Command::new("publish")
                .about("publish a message")
                .subcommand_value_name("message")
                .subcommands(publish_subcommands()),
            Command::new("get")
                .about("get message(s) from a queue")
                .subcommand_value_name("message")
                .subcommands(get_subcommands()),
        ])
}

fn list_subcommands() -> [Command; 15] {
    // duplicate this very common global argument so that
    // it can be passed as the end of argument list
    let vhost_arg = Arg::new("vhost")
        .short('V')
        .long("vhost")
        .help("target virtual host")
        .required(false)
        .default_value(DEFAULT_VHOST);

    [
        Command::new("nodes").long_about("Lists cluster members"),
        Command::new("users").long_about("Lists users in the internal database"),
        Command::new("vhosts")
            .long_about("Lists virtual hosts")
            .after_long_help(color_print::cstr!(
                "<bold>Doc guide</bold>: https://rabbitmq.com/vhosts.html"
            )),
        Command::new("permissions")
            .arg(vhost_arg.clone())
            .long_about("Lists user permissions")
            .after_long_help(color_print::cstr!(
                "<bold>Doc guide</bold>: https://rabbitmq.com/access-control.html"
            )),
        Command::new("connections")
            .arg(vhost_arg.clone())
            .long_about("Lists client connections")
            .after_long_help(color_print::cstr!(
                "<bold>Doc guide</bold>: https://rabbitmq.com/connections.html"
            )),
        Command::new("channels")
            .arg(vhost_arg.clone())
            .long_about("Lists AMQP 0-9-1 channels")
            .after_long_help(color_print::cstr!(
                "<bold>Doc guide</bold>: https://rabbitmq.com/channels.html"
            )),
        Command::new("queues")
            .arg(vhost_arg.clone())
            .long_about("Lists queues")
            .after_long_help(color_print::cstr!(
                "<bold>Doc guide</bold>: https://rabbitmq.com/queues.html"
            )),
        Command::new("exchanges").arg(vhost_arg.clone()),
        Command::new("bindings").arg(vhost_arg.clone()),
        Command::new("consumers")
            .arg(vhost_arg.clone())
            .long_about("Lists consumers")
            .after_long_help(color_print::cstr!(
                "<bold>Doc guide</bold>: https://rabbitmq.com/consumers.html"
            )),
        Command::new("parameters")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("component")
                    .long("component")
                    .help("component (for example: federation-upstream, vhost-limits)")
                    .required(false),
            )
            .long_about("Lists runtime parameters")
            .after_long_help(color_print::cstr!(
                "<bold>Doc guide</bold>: https://rabbitmq.com/parameters.html"
            )),
        Command::new("policies")
            .arg(vhost_arg.clone())
            .long_about("Lists policies")
            .after_long_help(color_print::cstr!(
                "<bold>Doc guide</bold>: https://rabbitmq.com/parameters.html"
            )),
        Command::new("operator_policies")
            .arg(vhost_arg.clone())
            .long_about("Lists operator policies")
            .after_long_help(color_print::cstr!(
                "<bold>Doc guide</bold>: https://rabbitmq.com/parameters.html"
            )),
        Command::new("vhost_limits")
            .arg(vhost_arg.clone())
            .long_about("Lists virtual host (resource) limits")
            .after_long_help(color_print::cstr!(
                "<bold>Doc guide</bold>: https://rabbitmq.com/vhosts.html"
            )),
        Command::new("user_limits")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("user")
                    .long("user")
                    .help("username")
                    .required(false),
            )
            .long_about("Lists per-user (resource) limits")
            .after_long_help(color_print::cstr!(
                "<bold>Doc guide</bold>: https://rabbitmq.com/user-limits.html"
            )),
    ]
}

fn declare_subcommands() -> [Command; 11] {
    // duplicate this very common global argument so that
    // it can be passed as the end of argument list
    let vhost_arg = Arg::new("vhost")
        .short('V')
        .long("vhost")
        .help("target virtual host")
        .required(false)
        .default_value(DEFAULT_VHOST);

    [
        Command::new("user")
            .about("creates a user")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("username")
                    .required(true),
            )
            .arg(
                Arg::new("password_hash")
                    .help("salted password hash, see https://rabbitmq.com/passwords.html")
                    .long("password_hash")
                    .required(false)
                    .default_value(""),
            )
            .arg(
                Arg::new("password")
                    .long("password")
                    .help("prefer providing a hash, see https://rabbitmq.com/passwords.html")
                    .required(false)
                    .default_value(""),
            )
            .arg(
                Arg::new("tags")
                    .long("tags")
                    .help("a list of comma-separated tags")
                    .default_value(""),
            ),
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
        Command::new("permissions")
            .about("grants permissions to a user")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("user")
                    .long("user")
                    .help("username")
                    .required(true),
            )
            .arg(
                Arg::new("configure")
                    .long("configure")
                    .help("name pattern for configuration access")
                    .required(true),
            )
            .arg(
                Arg::new("read")
                    .long("read")
                    .help("name pattern for read access")
                    .required(true),
            )
            .arg(
                Arg::new("write")
                    .long("write")
                    .help("name pattern for write access")
                    .required(true),
            ),
        Command::new("queue")
            .about("declares a queue")
            .arg(vhost_arg.clone())
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
            .arg(vhost_arg.clone())
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
        Command::new("binding")
            .about("binds to an exchange")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("source")
                    .long("source")
                    .help("source exchange")
                    .required(true),
            )
            .arg(
                Arg::new("destination_type")
                    .long("destination_type")
                    .help("destination type: exchange or queue")
                    .required(true)
                    .value_parser(clap::value_parser!(BindingDestinationType)),
            )
            .arg(
                Arg::new("destination")
                    .long("destination")
                    .help("destination exchange/queue name")
                    .required(true),
            )
            .arg(
                Arg::new("routing_key")
                    .long("routing_key")
                    .help("routing key")
                    .required(true),
            )
            .arg(
                Arg::new("arguments")
                    .long("arguments")
                    .help("additional arguments")
                    .required(false)
                    .default_value("{}")
                    .value_parser(clap::value_parser!(String)),
            ),
        Command::new("parameter").
            about("sets a runtime parameter")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("parameter's name")
                    .required(true)
            ).arg(
                Arg::new("component")
                    .long("component")
                    .help("component (eg. federation)")
                    .required(true))
            .arg(
                Arg::new("value")
                    .long("value")
                    .help("parameter's value")
                    .required(true)),
        Command::new("policy")
            .about("creates or updates a policy")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("policy name")
                    .required(true),
            )
            .arg(
                Arg::new("pattern")
                    .long("pattern")
                    .help("queue/exchange name pattern")
                    .required(true),
            )
            .arg(
                Arg::new("apply-to")
                    .long("apply-to")
                    .help("entities to apply to (queues, classic_queues, quorum_queues, streams, exchanges, all)")
                    .required(true),
            )
            .arg(
                Arg::new("priority")
                    .long("priority")
                    .help("policy priority (only the policy with the highest priority is effective)")
                    .required(false)
                    .default_value("0"),
            )
            .arg(
                Arg::new("definition")
                    .long("definition")
                    .help("policy definition")
                    .required(true),
            ),
        Command::new("operator_policy")
            .about("creates or updates an operator policy")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("operator policy name")
                    .required(true),
            )
            .arg(
                Arg::new("pattern")
                    .long("pattern")
                    .help("queue/exchange name pattern")
                    .required(true),
            )
            .arg(
                Arg::new("apply-to")
                    .long("apply-to")
                    .help("entities to apply to (queues, classic_queues, quorum_queues, streams, exchanges, all)")
                    .required(true),
            )
            .arg(
                Arg::new("priority")
                    .long("priority")
                    .help("policy priority (only the policy with the highest priority is effective)")
                    .required(false)
                    .default_value("0"),
            )
            .arg(
                Arg::new("definition")
                    .long("definition")
                    .help("policy definition")
                    .required(true),
            ),
        Command::new("vhost_limit")
            .about("set a vhost limit")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("limit name (eg. max-connections, max-queues)")
                    .required(true),
            )
            .arg(
                Arg::new("value")
                    .long("value")
                    .help("limit value")
                    .required(true),
            ),
        Command::new("user_limit").about("set a user limit")
            .arg(
                Arg::new("user")
                    .long("user")
                    .help("username")
                    .required(true),
            )
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("limit name (eg. max-connections, max-queues)")
                    .required(true),
            )
            .arg(
                Arg::new("value")
                    .long("value")
                    .help("limit value")
                    .required(true),
            )
    ]
}

fn show_subcomands() -> [Command; 1] {
    [Command::new("overview").about(
        "displays a subset of aggregated metrics found on the Overview page in management UI",
    )]
}

fn delete_subcommands() -> [Command; 11] {
    // duplicate this very common global argument so that
    // it can be passed as the end of argument list
    let vhost_arg = Arg::new("vhost")
        .short('V')
        .long("vhost")
        .help("target virtual host")
        .required(false)
        .default_value(DEFAULT_VHOST);

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
        Command::new("permissions")
            .about("revokes user permissions to a given vhost")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("user")
                    .long("user")
                    .help("username")
                    .required(true),
            ),
        Command::new("queue")
            .about("deletes a queue")
            .arg(vhost_arg.clone())
            .arg(Arg::new("name").long("name").help("queue").required(true)),
        Command::new("exchange")
            .about("deletes an exchange")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("exchange")
                    .required(true),
            ),
        Command::new("binding")
            .about("deletes a binding")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("source")
                    .long("source")
                    .help("source exchange")
                    .required(true),
            )
            .arg(
                Arg::new("destination_type")
                    .long("destination_type")
                    .help("destination type: exchange or queue")
                    .required(true),
            )
            .arg(
                Arg::new("destination")
                    .long("destination")
                    .help("destination exchange/queue name")
                    .required(true),
            )
            .arg(
                Arg::new("routing_key")
                    .long("routing_key")
                    .help("routing key")
                    .required(true),
            )
            .arg(
                Arg::new("arguments")
                    .long("arguments")
                    .help("additional arguments")
                    .required(false)
                    .default_value("{}")
                    .value_parser(clap::value_parser!(String)),
            ),
        Command::new("parameter")
            .about("clears a runtime parameter")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("parameter's name")
                    .required(true),
            )
            .arg(
                Arg::new("component")
                    .long("component")
                    .help("component (eg. federation-upstream)")
                    .required(true),
            ),
        Command::new("policy")
            .about("deletes a policy")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("policy name")
                    .required(true),
            ),
        Command::new("operator_policy")
            .about("deletes an operator policy")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("operator policy name")
                    .required(true),
            ),
        Command::new("vhost_limit")
            .about("delete a vhost limit")
            .arg(vhost_arg.clone())
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("limit name (eg. max-connections, max-queues)")
                    .required(true),
            ),
        Command::new("user_limit")
            .about("delete a user limit")
            .arg(
                Arg::new("user")
                    .long("user")
                    .help("username")
                    .required(true),
            )
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("limit name (eg. max-connections, max-queues)")
                    .required(true),
            ),
    ]
}

fn purge_subcommands() -> [Command; 1] {
    [Command::new("queue")
        .long_about("purges (permanently removes unacknowledged messages from) a queue")
        .arg(
            Arg::new("name")
                .long("name")
                .help("name of the queue to purge")
                .required(true),
        )]
}

fn rebalance_subcommands() -> [Command; 1] {
    [Command::new("queues").about("rebalances queue leaders")]
}

fn close_subcommands() -> [Command; 1] {
    [Command::new("connection")
        .about("closes a client connection")
        .arg(
            Arg::new("name")
                .long("name")
                .help("connection name (identifying string)")
                .required(true),
        )]
}

fn definitions_subcommands() -> [Command; 2] {
    let export_cmd = Command::new("export")
        .about("export all definitions (queues, exchanges, bindings, users, etc)")
        .after_long_help(color_print::cstr!(
            "<bold>Doc guide</bold>: https://rabbitmq.com/docs/definitions/"
        ))
        .arg(
            Arg::new("file")
                .long("file")
                .help("output path")
                .required(false)
                .default_value("-")
        );

    let import_cmd = Command::new("import")
        .about("import all definitions (queues, exchanges, bindings, users, etc) from a JSON file")
        .after_long_help(color_print::cstr!(
            "<bold>Doc guide</bold>: https://rabbitmq.com/docs/definitions/"
        )).arg(
        Arg::new("file")
            .long("file")
            .help("JSON file with definitions")
            .required(true));

    [export_cmd, import_cmd]
}

fn export_subcommands() -> [Command; 1] {
    [Command::new("definitions")
        .about("prefer 'definitions export'")
        .after_long_help(color_print::cstr!(
            "<bold>Doc guide</bold>: https://rabbitmq.com/docs/definitions/"
        ))
        .arg(
            Arg::new("file")
                .long("file")
                .help("output path")
                .required(false)
                .default_value("-"),
        )]
}

fn import_subcommands() -> [Command; 1] {
    [Command::new("definitions")
        .about("prefer 'definitions import'")
        .after_long_help(color_print::cstr!(
            "<bold>Doc guide</bold>: https://rabbitmq.com/docs/definitions/"
        ))
        .arg(
            Arg::new("file")
                .long("file")
                .help("JSON file with definitions")
                .required(true),
        )]
}

pub fn publish_subcommands() -> [Command; 1] {
    [Command::new("message")
        .about("Publishes a message to an exchange")
        .arg(
            Arg::new("routing-key")
                .short('k')
                .long("routing-key")
                .required(false)
                .default_value("")
                .help("Name of virtual host"),
        )
        .arg(
            Arg::new("exchange")
                .short('e')
                .long("exchange")
                .required(false)
                .default_value("")
                .help("Exchange name (defaults to empty)"),
        )
        .arg(
            Arg::new("payload")
                .short('m')
                .long("payload")
                .required(false)
                .default_value("test")
                .help("Message payload/body"),
        )
        .arg(
            Arg::new("properties")
                .short('p')
                .long("properties")
                .required(false)
                .default_value("{}")
                .help("Message properties"),
        )]
}

pub fn get_subcommands() -> [Command; 1] {
    [Command::new("messages")
        .about("Consumes message(s) from a queue")
        .arg(
            Arg::new("queue")
                .short('q')
                .long("queue")
                .required(true)
                .help("Queue name"),
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .required(false)
                .default_value("1")
                .help("Maximum number of messages to consume"),
        )
        .arg(
            Arg::new("ack-mode")
                .short('a')
                .long("ack-mode")
                .required(false)
                .default_value("ack_requeue_false")
                .help("ack_requeue_false, reject_requeue_false, ack_requeue_true or reject_requeue_true"),
        )]
}
