use clap::ArgMatches;
use rabbitmq_http_client::commons;
use rabbitmq_http_client::commons::BindingDestinationType;
use rabbitmq_http_client::commons::UserLimitTarget;
use rabbitmq_http_client::commons::VirtualHostLimitTarget;
use rabbitmq_http_client::requests::EnforcedLimitParams;
use std::process;

use rabbitmq_http_client::blocking::Client as APIClient;
use rabbitmq_http_client::blocking::Result as ClientResult;
use rabbitmq_http_client::commons::QueueType;
use rabbitmq_http_client::{password_hashing, requests, responses};

use crate::cli::SharedFlags;

pub fn list_nodes(general_args: &ArgMatches) -> ClientResult<Vec<responses::ClusterNode>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.list_nodes()
}

pub fn list_vhosts(general_args: &ArgMatches) -> ClientResult<Vec<responses::VirtualHost>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.list_vhosts()
}

pub fn list_vhost_limits(
    general_args: &ArgMatches,
) -> ClientResult<Vec<responses::VirtualHostLimits>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.list_vhost_limits(&sf.virtual_host)
}

pub fn list_user_limits(
    general_args: &ArgMatches,
    command_args: &ArgMatches,
) -> ClientResult<Vec<responses::UserLimits>> {
    let sf = SharedFlags::from_args(general_args);
    let user = command_args.get_one::<String>("user");
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    match user {
        None => rc.list_all_user_limits(),
        Some(username) => rc.list_user_limits(username),
    }
}

pub fn list_users(general_args: &ArgMatches) -> ClientResult<Vec<responses::User>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.list_users()
}

pub fn list_connections(general_args: &ArgMatches) -> ClientResult<Vec<responses::Connection>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.list_connections()
}

pub fn list_channels(general_args: &ArgMatches) -> ClientResult<Vec<responses::Channel>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.list_channels()
}

pub fn list_consumers(general_args: &ArgMatches) -> ClientResult<Vec<responses::Consumer>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.list_consumers()
}

pub fn list_policies(general_args: &ArgMatches) -> ClientResult<Vec<responses::Policy>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.list_policies()
}

pub fn list_operator_policies(general_args: &ArgMatches) -> ClientResult<Vec<responses::Policy>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.list_operator_policies()
}

pub fn list_queues(general_args: &ArgMatches) -> ClientResult<Vec<responses::QueueInfo>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.list_queues_in(&sf.virtual_host)
}

pub fn list_exchanges(general_args: &ArgMatches) -> ClientResult<Vec<responses::ExchangeInfo>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.list_exchanges_in(&sf.virtual_host)
}

pub fn list_bindings(general_args: &ArgMatches) -> ClientResult<Vec<responses::BindingInfo>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.list_bindings()
}

pub fn list_permissions(general_args: &ArgMatches) -> ClientResult<Vec<responses::Permissions>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.list_permissions()
}

pub fn list_parameters(
    general_args: &ArgMatches,
) -> ClientResult<Vec<responses::RuntimeParameter>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.list_runtime_parameters()
}

pub fn declare_vhost(general_args: &ArgMatches, command_args: &ArgMatches) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    // these are optional
    let description = command_args
        .get_one::<String>("description")
        .map(|s| s.as_str());
    let dqt = command_args
        .get_one::<String>("default_queue_type")
        .map(|s| Into::<QueueType>::into(s.as_str()));
    // TODO: tags
    let tracing = command_args.get_one::<bool>("tracing").unwrap_or(&false);

    let params = requests::VirtualHostParams {
        name,
        description,
        default_queue_type: dqt,
        tags: None,
        tracing: *tracing,
    };

    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.create_vhost(&params)
}

pub fn declare_exchange(general_args: &ArgMatches, command_args: &ArgMatches) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    // these are optional
    let exchange_type = command_args
        .get_one::<String>("type")
        .map(|s| Into::<rabbitmq_http_client::commons::ExchangeType>::into(s.as_str()))
        .unwrap_or(rabbitmq_http_client::commons::ExchangeType::Direct);
    let durable = command_args.get_one::<bool>("durable").unwrap_or(&true);
    let auto_delete = command_args
        .get_one::<bool>("auto_delete")
        .unwrap_or(&false);
    let arguments = command_args.get_one::<String>("arguments").unwrap();

    let params = requests::ExchangeParams {
        name,
        exchange_type,
        durable: *durable,
        auto_delete: *auto_delete,
        arguments: serde_json::from_str::<requests::XArguments>(arguments).unwrap_or_else(|err| {
            eprintln!("`{}` is not a valid JSON: {}", arguments, err);
            process::exit(1);
        }),
    };

    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.declare_exchange(&sf.virtual_host, &params)
}

pub fn declare_binding(general_args: &ArgMatches, command_args: &ArgMatches) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);

    let source = command_args.get_one::<String>("source").unwrap();
    let destination_type = command_args
        .get_one::<BindingDestinationType>("destination_type")
        .unwrap();
    let destination = command_args.get_one::<String>("destination").unwrap();
    let routing_key = command_args.get_one::<String>("routing_key").unwrap();
    let arguments = command_args.get_one::<String>("arguments").unwrap();
    let parsed_arguments =
        serde_json::from_str::<requests::XArguments>(arguments).unwrap_or_else(|err| {
            eprintln!("`{}` is not a valid JSON: {}", arguments, err);
            process::exit(1);
        });

    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    match destination_type {
        BindingDestinationType::Queue => rc.bind_queue(
            &sf.virtual_host,
            destination,
            source,
            Some(routing_key),
            parsed_arguments,
        ),
        BindingDestinationType::Exchange => rc.bind_exchange(
            &sf.virtual_host,
            destination,
            source,
            Some(routing_key),
            parsed_arguments,
        ),
    }
}

pub fn declare_vhost_limit(
    general_args: &ArgMatches,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);

    let name = command_args.get_one::<String>("name").unwrap();
    let value = command_args.get_one::<String>("value").unwrap();

    let limit = EnforcedLimitParams::new(
        VirtualHostLimitTarget::from(name.as_str()),
        str::parse(value).unwrap(),
    );

    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.set_vhost_limit(&sf.virtual_host, limit)
}

pub fn declare_user_limit(
    general_args: &ArgMatches,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);

    let user = command_args.get_one::<String>("user").unwrap();
    let name = command_args.get_one::<String>("name").unwrap();
    let value = command_args.get_one::<String>("value").unwrap();

    let limit = EnforcedLimitParams::new(
        UserLimitTarget::from(name.as_str()),
        str::parse(value).unwrap(),
    );

    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.set_user_limit(user, limit)
}

pub fn delete_vhost_limit(
    general_args: &ArgMatches,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);

    let name = command_args.get_one::<String>("name").unwrap();

    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.clear_vhost_limit(
        &sf.virtual_host,
        VirtualHostLimitTarget::from(name.as_str()),
    )
}

pub fn delete_user_limit(general_args: &ArgMatches, command_args: &ArgMatches) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);

    let user = command_args.get_one::<String>("user").unwrap();
    let name = command_args.get_one::<String>("name").unwrap();

    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.clear_user_limit(user, UserLimitTarget::from(name.as_str()))
}

pub fn delete_vhost(general_args: &ArgMatches, command_args: &ArgMatches) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.delete_vhost(name)
}

pub fn delete_user(general_args: &ArgMatches, command_args: &ArgMatches) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.delete_user(name)
}

pub fn delete_permissions(
    general_args: &ArgMatches,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);
    // the flag is required
    let user = command_args.get_one::<String>("user").unwrap();
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.delete_permissions(&sf.virtual_host, user)
}

pub fn declare_user(general_args: &ArgMatches, command_args: &ArgMatches) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);
    let name = command_args.get_one::<String>("name").unwrap();
    let password = command_args.get_one::<String>("password").unwrap();
    let provided_hash = command_args.get_one::<String>("password_hash").unwrap();
    let tags = command_args.get_one::<String>("tags").unwrap();
    let endpoint = sf.endpoint();

    if password.is_empty() && provided_hash.is_empty()
        || !password.is_empty() && !provided_hash.is_empty()
    {
        eprintln!("Please provide either --password or --password_hash");
        process::exit(1)
    }

    let password_hash = if provided_hash.is_empty() {
        let salt = password_hashing::salt();
        password_hashing::base64_encoded_salted_password_hash_sha256(&salt, password)
    } else {
        provided_hash.to_string()
    };

    let params = requests::UserParams {
        name,
        password_hash: password_hash.as_str(),
        tags,
    };
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.create_user(&params)
}

pub fn declare_permissions(
    general_args: &ArgMatches,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);
    let user = command_args.get_one::<String>("user").unwrap();
    let configure = command_args.get_one::<String>("configure").unwrap();
    let read = command_args.get_one::<String>("read").unwrap();
    let write = command_args.get_one::<String>("write").unwrap();

    let params = requests::Permissions {
        user,
        vhost: &sf.virtual_host,
        configure,
        read,
        write,
    };

    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.declare_permissions(&params)
}

pub fn declare_queue(general_args: &ArgMatches, command_args: &ArgMatches) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    let queue_type = command_args.get_one::<QueueType>("type").unwrap();
    // these are optional
    let durable = command_args.get_one::<bool>("durable").unwrap_or(&true);
    let auto_delete = command_args
        .get_one::<bool>("auto_delete")
        .unwrap_or(&false);
    let arguments = command_args.get_one::<String>("arguments").unwrap();

    let parsed_args =
        serde_json::from_str::<requests::XArguments>(arguments).unwrap_or_else(|err| {
            eprintln!("`{}` is not a valid JSON: {}", arguments, err);
            process::exit(1);
        });

    let params = requests::QueueParams::new(name, *queue_type, *durable, *auto_delete, parsed_args);

    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.declare_queue(&sf.virtual_host, &params)
}

pub fn declare_policy(general_args: &ArgMatches, command_args: &ArgMatches) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);

    let name = command_args.get_one::<String>("name").unwrap();
    let pattern = command_args.get_one::<String>("pattern").unwrap();
    let apply_to = command_args.get_one::<String>("pattern").unwrap();
    let priority = command_args.get_one::<String>("priority").unwrap();
    let definition = command_args.get_one::<String>("definition").unwrap();

    let parsed_definition = serde_json::from_str::<requests::PolicyDefinition>(definition)
        .unwrap_or_else(|err| {
            eprintln!("`{}` is not a valid JSON: {}", definition, err);
            process::exit(1);
        });

    let params = requests::PolicyParams {
        vhost: &sf.virtual_host,
        name,
        pattern,
        apply_to: commons::PolicyTarget::from(apply_to.as_str()),
        priority: priority.parse::<i32>().unwrap(),
        definition: parsed_definition,
    };

    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.declare_policy(&params)
}

pub fn declare_operator_policy(
    general_args: &ArgMatches,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);

    let name = command_args.get_one::<String>("name").unwrap();
    let pattern = command_args.get_one::<String>("pattern").unwrap();
    let apply_to = command_args.get_one::<String>("pattern").unwrap();
    let priority = command_args.get_one::<String>("priority").unwrap();
    let definition = command_args.get_one::<String>("definition").unwrap();

    let parsed_definition = serde_json::from_str::<requests::PolicyDefinition>(definition)
        .unwrap_or_else(|err| {
            eprintln!("`{}` is not a valid JSON: {}", definition, err);
            process::exit(1);
        });

    let params = requests::PolicyParams {
        vhost: &sf.virtual_host,
        name,
        pattern,
        apply_to: commons::PolicyTarget::from(apply_to.as_str()),
        priority: priority.parse::<i32>().unwrap(),
        definition: parsed_definition,
    };

    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.declare_operator_policy(&params)
}

pub fn delete_queue(general_args: &ArgMatches, command_args: &ArgMatches) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.delete_queue(&sf.virtual_host, name)
}

pub fn delete_binding(general_args: &ArgMatches, command_args: &ArgMatches) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);

    let source = command_args.get_one::<String>("source").unwrap();
    let destination_type = command_args.get_one::<String>("destination_type").unwrap();
    let destination = command_args.get_one::<String>("destination").unwrap();
    let routing_key = command_args.get_one::<String>("routing_key").unwrap();
    let arguments = command_args.get_one::<String>("arguments").unwrap();
    let parsed_arguments =
        serde_json::from_str::<requests::XArguments>(arguments).unwrap_or_else(|err| {
            eprintln!("`{}` is not a valid JSON: {}", arguments, err);
            process::exit(1);
        });

    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.delete_binding(
        &sf.virtual_host,
        source,
        destination,
        BindingDestinationType::from(destination_type.clone()),
        Some(routing_key),
        parsed_arguments,
    )
    .map(|_| ())
}

pub fn delete_exchange(general_args: &ArgMatches, command_args: &ArgMatches) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.delete_exchange(&sf.virtual_host, name)
}

pub fn delete_policy(general_args: &ArgMatches, command_args: &ArgMatches) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.delete_policy(&sf.virtual_host, name)
}

pub fn delete_operator_policy(
    general_args: &ArgMatches,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.delete_operator_policy(&sf.virtual_host, name)
}

pub fn purge_queue(general_args: &ArgMatches, command_args: &ArgMatches) -> ClientResult<()> {
    let sf = SharedFlags::from_args(general_args);
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    let endpoint = sf.endpoint();
    let rc = APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, &sf.password);
    rc.purge_queue(&sf.virtual_host, name)
}
