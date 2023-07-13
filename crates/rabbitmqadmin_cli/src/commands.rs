use clap::ArgMatches;
use rabbitmq_http_client::commons;
use rabbitmq_http_client::commons::UserLimitTarget;
use rabbitmq_http_client::commons::VirtualHostLimitTarget;
use std::process;

use rabbitmq_http_client::blocking::Client as APIClient;
use rabbitmq_http_client::blocking::Result as ClientResult;
use rabbitmq_http_client::requests::EnforcedLimitParams;

use rabbitmq_http_client::commons::BindingDestinationType;
use rabbitmq_http_client::commons::QueueType;
use rabbitmq_http_client::{password_hashing, requests, responses};

pub fn list_nodes(client: APIClient) -> ClientResult<Vec<responses::ClusterNode>> {
    client.list_nodes()
}

pub fn list_vhosts(client: APIClient) -> ClientResult<Vec<responses::VirtualHost>> {
    client.list_vhosts()
}

pub fn list_vhost_limits(
    client: APIClient,
    vhost: &str,
) -> ClientResult<Vec<responses::VirtualHostLimits>> {
    client.list_vhost_limits(vhost)
}

pub fn list_user_limits(
    client: APIClient,
    command_args: &ArgMatches,
) -> ClientResult<Vec<responses::UserLimits>> {
    let user = command_args.get_one::<String>("user");
    match user {
        None => client.list_all_user_limits(),
        Some(username) => client.list_user_limits(username),
    }
}

pub fn list_users(client: APIClient) -> ClientResult<Vec<responses::User>> {
    client.list_users()
}

pub fn list_connections(client: APIClient) -> ClientResult<Vec<responses::Connection>> {
    client.list_connections()
}

pub fn list_channels(client: APIClient) -> ClientResult<Vec<responses::Channel>> {
    client.list_channels()
}

pub fn list_consumers(client: APIClient) -> ClientResult<Vec<responses::Consumer>> {
    client.list_consumers()
}

pub fn list_policies(client: APIClient) -> ClientResult<Vec<responses::Policy>> {
    client.list_policies()
}

pub fn list_operator_policies(client: APIClient) -> ClientResult<Vec<responses::Policy>> {
    client.list_operator_policies()
}

pub fn list_queues(client: APIClient, vhost: &str) -> ClientResult<Vec<responses::QueueInfo>> {
    client.list_queues_in(vhost)
}

pub fn list_exchanges(
    client: APIClient,
    vhost: &str,
) -> ClientResult<Vec<responses::ExchangeInfo>> {
    client.list_exchanges_in(vhost)
}

pub fn list_bindings(client: APIClient) -> ClientResult<Vec<responses::BindingInfo>> {
    client.list_bindings()
}

pub fn list_permissions(client: APIClient) -> ClientResult<Vec<responses::Permissions>> {
    client.list_permissions()
}

pub fn list_parameters(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<Vec<responses::RuntimeParameter>> {
    let component = command_args.get_one::<String>("component");
    match component {
        None => {
            let mut r = client.list_runtime_parameters()?;
            r.retain(|p| p.vhost == vhost);
            Ok(r)
        }
        Some(c) => client.list_runtime_parameters_of_component_in(c, vhost),
    }
}

pub fn declare_vhost(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
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

    client.create_vhost(&params)
}

pub fn declare_exchange(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
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

    client.declare_exchange(vhost, &params)
}

pub fn declare_binding(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
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

    match destination_type {
        BindingDestinationType::Queue => client.bind_queue(
            vhost,
            destination,
            source,
            Some(routing_key),
            parsed_arguments,
        ),
        BindingDestinationType::Exchange => client.bind_exchange(
            vhost,
            destination,
            source,
            Some(routing_key),
            parsed_arguments,
        ),
    }
}

pub fn declare_vhost_limit(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let name = command_args.get_one::<String>("name").unwrap();
    let value = command_args.get_one::<String>("value").unwrap();

    let limit = EnforcedLimitParams::new(
        VirtualHostLimitTarget::from(name.as_str()),
        str::parse(value).unwrap(),
    );

    client.set_vhost_limit(vhost, limit)
}

pub fn declare_user_limit(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    let user = command_args.get_one::<String>("user").unwrap();
    let name = command_args.get_one::<String>("name").unwrap();
    let value = command_args.get_one::<String>("value").unwrap();

    let limit = EnforcedLimitParams::new(
        UserLimitTarget::from(name.as_str()),
        str::parse(value).unwrap(),
    );

    client.set_user_limit(user, limit)
}

pub fn delete_vhost_limit(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let name = command_args.get_one::<String>("name").unwrap();

    client.clear_vhost_limit(vhost, VirtualHostLimitTarget::from(name.as_str()))
}

pub fn delete_user_limit(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    let user = command_args.get_one::<String>("user").unwrap();
    let name = command_args.get_one::<String>("name").unwrap();

    client.clear_user_limit(user, UserLimitTarget::from(name.as_str()))
}

pub fn delete_parameter(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let component = command_args.get_one::<String>("component").unwrap();
    let name = command_args.get_one::<String>("name").unwrap();

    client.clear_runtime_parameter(component, vhost, name)
}

pub fn delete_vhost(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    client.delete_vhost(name)
}

pub fn delete_user(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    client.delete_user(name)
}

pub fn delete_permissions(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    // the flag is required
    let user = command_args.get_one::<String>("user").unwrap();
    client.clear_permissions(vhost, user)
}

pub fn declare_user(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    let name = command_args.get_one::<String>("name").unwrap();
    let password = command_args.get_one::<String>("password").unwrap();
    let provided_hash = command_args.get_one::<String>("password_hash").unwrap();
    let tags = command_args.get_one::<String>("tags").unwrap();

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
    client.create_user(&params)
}

pub fn declare_permissions(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let user = command_args.get_one::<String>("user").unwrap();
    let configure = command_args.get_one::<String>("configure").unwrap();
    let read = command_args.get_one::<String>("read").unwrap();
    let write = command_args.get_one::<String>("write").unwrap();

    let params = requests::Permissions {
        user,
        vhost,
        configure,
        read,
        write,
    };

    client.declare_permissions(&params)
}

pub fn declare_queue(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
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

    client.declare_queue(vhost, &params)
}

pub fn declare_policy(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
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
        vhost,
        name,
        pattern,
        apply_to: commons::PolicyTarget::from(apply_to.as_str()),
        priority: priority.parse::<i32>().unwrap(),
        definition: parsed_definition,
    };

    client.declare_policy(&params)
}

pub fn declare_operator_policy(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
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
        vhost,
        name,
        pattern,
        apply_to: commons::PolicyTarget::from(apply_to.as_str()),
        priority: priority.parse::<i32>().unwrap(),
        definition: parsed_definition,
    };

    client.declare_operator_policy(&params)
}

pub fn declare_parameter(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let component = command_args.get_one::<String>("component").unwrap();
    let name = command_args.get_one::<String>("name").unwrap();
    let value = command_args.get_one::<String>("value").unwrap();
    let parsed_value = serde_json::from_str::<requests::RuntimeParameterValue>(value)
        .unwrap_or_else(|err| {
            eprintln!("`{}` is not a valid JSON: {}", value, err);
            process::exit(1);
        });

    let params = requests::RuntimeParameterDefinition {
        vhost: vhost.to_string(),
        name: name.to_owned(),
        component: component.to_owned(),
        value: parsed_value,
    };

    client.upsert_runtime_parameter(&params)
}

pub fn delete_queue(client: APIClient, vhost: &str, command_args: &ArgMatches) -> ClientResult<()> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    client.delete_queue(vhost, name)
}

pub fn delete_binding(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
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

    client
        .delete_binding(
            vhost,
            source,
            destination,
            BindingDestinationType::from(destination_type.clone()),
            routing_key,
            parsed_arguments,
        )
        .map(|_| ())
}

pub fn delete_exchange(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    client.delete_exchange(vhost, name)
}

pub fn delete_policy(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    client.delete_policy(vhost, name)
}

pub fn delete_operator_policy(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    client.delete_operator_policy(vhost, name)
}

pub fn purge_queue(client: APIClient, vhost: &str, command_args: &ArgMatches) -> ClientResult<()> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    client.purge_queue(vhost, name)
}

pub fn close_connection(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    client.close_connection(name, Some("closed via rabbitmqadmin v2"))
}

pub fn rebalance_queues(client: APIClient) -> ClientResult<()> {
    client.rebalance_queue_leaders()
}
