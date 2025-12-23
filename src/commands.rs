// Copyright (C) 2023-2025 RabbitMQ Core Team (teamrabbitmq@gmail.com)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#![allow(clippy::result_large_err)]

use crate::arg_helpers::ArgMatchesExt;
use crate::config::{
    ConfigPathEntry, NodeConfigEntry, Scheme, SharedSettings, add_node_to_config_file,
    config_file_exists, delete_node_from_config_file, list_all_nodes, update_node_in_config_file,
};
use crate::constants::{DEFAULT_BLANKET_POLICY_PRIORITY, DEFAULT_HOST, DEFAULT_VHOST};
use crate::errors::CommandRunError;
use crate::output::ProgressReporter;
use crate::pre_flight;
use clap::ArgMatches;
use rabbitmq_http_client::blocking_api::Client;
use rabbitmq_http_client::commons;
use rabbitmq_http_client::commons::PaginationParams;
use rabbitmq_http_client::commons::QueueType;
use rabbitmq_http_client::commons::{
    BindingDestinationType, ChannelUseMode, TlsPeerVerificationMode,
};
use rabbitmq_http_client::commons::{ExchangeType, SupportedProtocol};
use rabbitmq_http_client::commons::{MessageTransferAcknowledgementMode, UserLimitTarget};
use rabbitmq_http_client::commons::{PolicyTarget, VirtualHostLimitTarget};
use rabbitmq_http_client::password_hashing::{HashingAlgorithm, HashingError};
use rabbitmq_http_client::requests::shovels::OwnedShovelParams;
use rabbitmq_http_client::requests::{
    Amqp10ShovelDestinationParams, Amqp10ShovelParams, Amqp10ShovelSourceParams,
    Amqp091ShovelDestinationParams, Amqp091ShovelParams, Amqp091ShovelSourceParams,
    BindingDeletionParams, EnforcedLimitParams, ExchangeFederationParams,
    FEDERATION_UPSTREAM_COMPONENT, FederationResourceCleanupMode, FederationUpstreamParams,
    OwnedFederationUpstreamParams, PolicyParams, QueueFederationParams, RuntimeParameterDefinition,
};

use rabbitmq_http_client::transformers::{TransformationChain, VirtualHostTransformationChain};
use rabbitmq_http_client::uris::UriBuilder;
use rabbitmq_http_client::{password_hashing, requests, responses};
use regex::Regex;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tabled::Tabled;

type APIClient = Client<String, String, String>;
type CommandResult<T> = Result<T, CommandRunError>;

/// Certificate paths for TLS peer verification operations
pub struct CertPaths<'a> {
    pub ca_cert_path: &'a str,
    pub client_cert_path: &'a str,
    pub client_key_path: &'a str,
}

impl<'a> CertPaths<'a> {
    pub fn from_args(args: &'a ArgMatches) -> Result<Self, CommandRunError> {
        let ca_cert_path = args
            .get_one::<String>("node_local_ca_certificate_bundle_path")
            .map(String::as_str)
            .ok_or_else(|| CommandRunError::MissingArgumentValue {
                property: "node_local_ca_certificate_bundle_path".to_string(),
            })?;
        let client_cert_path = args
            .get_one::<String>("node_local_client_certificate_file_path")
            .map(String::as_str)
            .ok_or_else(|| CommandRunError::MissingArgumentValue {
                property: "node_local_client_certificate_file_path".to_string(),
            })?;
        let client_key_path = args
            .get_one::<String>("node_local_client_private_key_file_path")
            .map(String::as_str)
            .ok_or_else(|| CommandRunError::MissingArgumentValue {
                property: "node_local_client_private_key_file_path".to_string(),
            })?;

        Ok(Self {
            ca_cert_path,
            client_cert_path,
            client_key_path,
        })
    }
}

/// Specifies which URI field to update in shovel parameters
#[derive(Clone, Copy)]
pub enum ShovelUriField {
    Source,
    Destination,
}

impl ShovelUriField {
    fn operation_name(self) -> &'static str {
        match self {
            ShovelUriField::Source => "Updating shovel source URIs",
            ShovelUriField::Destination => "Updating shovel destination URIs",
        }
    }

    fn empty_uri_reason(self) -> &'static str {
        match self {
            ShovelUriField::Source => "empty source URI",
            ShovelUriField::Destination => "empty destination URI",
        }
    }
}

fn update_all_federation_upstream_uris<F>(
    client: &APIClient,
    prog_rep: &mut dyn ProgressReporter,
    uri_transformer: F,
) -> Result<(), CommandRunError>
where
    F: Fn(&str) -> Result<String, CommandRunError>,
{
    let upstreams = client.list_federation_upstreams()?;
    let total = upstreams.len();
    prog_rep.start_operation(total, "Updating federation upstream URIs");

    for (index, upstream) in upstreams.into_iter().enumerate() {
        let upstream_name = upstream.name.clone();
        prog_rep.report_progress(index + 1, total, &upstream_name);

        let updated_uri = uri_transformer(&upstream.uri)?;
        let owned_params = OwnedFederationUpstreamParams::from(upstream).with_uri(updated_uri);
        let upstream_params = FederationUpstreamParams::from(&owned_params);

        let param = RuntimeParameterDefinition::from(upstream_params);
        client.upsert_runtime_parameter(&param)?;
        prog_rep.report_success(&upstream_name);
    }

    prog_rep.finish_operation(total);
    Ok(())
}

fn update_all_shovel_uris<F>(
    client: &APIClient,
    prog_rep: &mut dyn ProgressReporter,
    field: ShovelUriField,
    uri_transformer: F,
) -> Result<(), CommandRunError>
where
    F: Fn(&str) -> Result<String, CommandRunError>,
{
    let all_params = client.list_runtime_parameters()?;
    let shovel_params: Vec<_> = all_params.into_iter().filter(|p| p.is_shovel()).collect();

    let total = shovel_params.len();
    prog_rep.start_operation(total, field.operation_name());

    for (index, param) in shovel_params.into_iter().enumerate() {
        let param_name = &param.name;
        prog_rep.report_progress(index + 1, total, param_name);

        let mut owned_params = match OwnedShovelParams::try_from(param.clone()) {
            Ok(params) => params,
            Err(_) => {
                prog_rep.report_skip(param_name, "shovel parameters fail validation");
                continue;
            }
        };

        let original_uri = match field {
            ShovelUriField::Source => &owned_params.source_uri,
            ShovelUriField::Destination => &owned_params.destination_uri,
        };

        if original_uri.is_empty() {
            prog_rep.report_skip(param_name, field.empty_uri_reason());
            continue;
        }

        let updated_uri = uri_transformer(original_uri)?;

        match field {
            ShovelUriField::Source => owned_params.source_uri = updated_uri,
            ShovelUriField::Destination => owned_params.destination_uri = updated_uri,
        }

        let param = RuntimeParameterDefinition::from(&owned_params);
        client.upsert_runtime_parameter(&param)?;
        prog_rep.report_success(param_name);
    }

    prog_rep.finish_operation(total);
    Ok(())
}

pub fn show_overview(client: APIClient) -> CommandResult<responses::Overview> {
    Ok(client.overview()?)
}

pub fn show_memory_breakdown(
    client: APIClient,
    command_args: &ArgMatches,
) -> CommandResult<Option<responses::NodeMemoryBreakdown>> {
    let node = command_args.str_arg("node");
    Ok(client
        .get_node_memory_footprint(node)
        .map(|footprint| footprint.breakdown)?)
}

pub fn list_nodes(client: APIClient) -> CommandResult<Vec<responses::ClusterNode>> {
    Ok(client.list_nodes()?)
}

pub fn list_auth_attempts(
    client: APIClient,
    command_args: &ArgMatches,
) -> CommandResult<Vec<responses::AuthenticationAttemptStatistics>> {
    let node = command_args.str_arg("node");
    Ok(client.auth_attempts_statistics(node)?)
}

pub fn list_vhosts(client: APIClient) -> CommandResult<Vec<responses::VirtualHost>> {
    Ok(client.list_vhosts()?)
}

pub fn list_vhost_limits(
    client: APIClient,
    vhost: &str,
) -> CommandResult<Vec<responses::VirtualHostLimits>> {
    Ok(client.list_vhost_limits(vhost)?)
}

pub fn list_user_limits(
    client: APIClient,
    command_args: &ArgMatches,
) -> CommandResult<Vec<responses::UserLimits>> {
    match command_args.optional_string("user") {
        None => Ok(client.list_all_user_limits()?),
        Some(username) => Ok(client.list_user_limits(&username)?),
    }
}

pub fn list_users(
    client: APIClient,
    command_args: &ArgMatches,
) -> CommandResult<Vec<responses::User>> {
    let pagination = extract_pagination_params(command_args);
    match pagination {
        Some(params) => Ok(client.list_users_paged(&params)?),
        None => Ok(client.list_users()?),
    }
}

pub fn list_connections(
    client: APIClient,
    command_args: &ArgMatches,
) -> CommandResult<Vec<responses::Connection>> {
    let pagination = extract_pagination_params(command_args);
    match pagination {
        Some(params) => Ok(client.list_connections_paged(&params)?),
        None => Ok(client.list_connections()?),
    }
}

fn extract_pagination_params(command_args: &ArgMatches) -> Option<PaginationParams> {
    let page = command_args
        .get_one::<u64>("page")
        .copied()
        .map(|v| v as usize);
    let page_size = command_args
        .get_one::<u64>("page_size")
        .copied()
        .map(|v| v as usize);
    if page.is_some() || page_size.is_some() {
        Some(PaginationParams { page, page_size })
    } else {
        None
    }
}

pub fn list_user_connections(
    client: APIClient,
    command_args: &ArgMatches,
) -> CommandResult<Vec<responses::UserConnection>> {
    let username = command_args.string_arg("username");
    Ok(client.list_user_connections(&username)?)
}

pub fn list_channels(
    client: APIClient,
    command_args: &ArgMatches,
) -> CommandResult<Vec<responses::Channel>> {
    let pagination = extract_pagination_params(command_args);
    match pagination {
        Some(params) => Ok(client.list_channels_paged(&params)?),
        None => Ok(client.list_channels()?),
    }
}

pub fn list_consumers(client: APIClient) -> CommandResult<Vec<responses::Consumer>> {
    Ok(client.list_consumers()?)
}

pub fn list_policies(client: APIClient) -> CommandResult<Vec<responses::Policy>> {
    Ok(client.list_policies()?)
}

pub fn list_policies_in(client: APIClient, vhost: &str) -> CommandResult<Vec<responses::Policy>> {
    Ok(client.list_policies_in(vhost)?)
}

pub fn list_policies_in_and_applying_to(
    client: APIClient,
    vhost: &str,
    apply_to: PolicyTarget,
) -> CommandResult<Vec<responses::Policy>> {
    Ok(client.list_policies_for_target(vhost, apply_to)?)
}

pub fn list_matching_policies_in(
    client: APIClient,
    vhost: &str,
    name: &str,
    typ: PolicyTarget,
) -> CommandResult<Vec<responses::Policy>> {
    Ok(client.list_matching_policies(vhost, name, typ)?)
}

pub fn list_policies_with_conflicting_priorities(
    client: APIClient,
) -> CommandResult<Vec<responses::Policy>> {
    let policies = client.list_policies()?;
    Ok(filter_policies_with_conflicting_priorities(policies))
}

pub fn list_policies_with_conflicting_priorities_in(
    client: APIClient,
    vhost: &str,
) -> CommandResult<Vec<responses::Policy>> {
    let policies = client.list_policies_in(vhost)?;
    Ok(filter_policies_with_conflicting_priorities(policies))
}

fn filter_policies_with_conflicting_priorities(
    policies: Vec<responses::Policy>,
) -> Vec<responses::Policy> {
    let mut priority_counts: HashMap<(&str, i16), usize> = HashMap::new();

    for pol in &policies {
        *priority_counts
            .entry((pol.vhost.as_str(), pol.priority))
            .or_insert(0) += 1;
    }

    let dominated: Vec<bool> = policies
        .iter()
        .map(|pol| {
            priority_counts
                .get(&(pol.vhost.as_str(), pol.priority))
                .is_some_and(|&count| count > 1)
        })
        .collect();

    policies
        .into_iter()
        .zip(dominated)
        .filter_map(|(pol, is_conflicting)| is_conflicting.then_some(pol))
        .collect()
}

pub fn list_operator_policies(client: APIClient) -> CommandResult<Vec<responses::Policy>> {
    Ok(client.list_operator_policies()?)
}

pub fn list_operator_policies_in(
    client: APIClient,
    vhost: &str,
) -> CommandResult<Vec<responses::Policy>> {
    Ok(client.list_operator_policies_in(vhost)?)
}

pub fn list_operator_policies_in_and_applying_to(
    client: APIClient,
    vhost: &str,
    apply_to: PolicyTarget,
) -> CommandResult<Vec<responses::Policy>> {
    Ok(client.list_operator_policies_for_target(vhost, apply_to)?)
}

pub fn list_matching_operator_policies_in(
    client: APIClient,
    vhost: &str,
    name: &str,
    typ: PolicyTarget,
) -> CommandResult<Vec<responses::Policy>> {
    Ok(client.list_matching_operator_policies(vhost, name, typ)?)
}

pub fn list_queues(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<Vec<responses::QueueInfo>> {
    let pagination = extract_pagination_params(command_args);
    match pagination {
        Some(params) => Ok(client.list_queues_in_paged(vhost, &params)?),
        None => Ok(client.list_queues_in(vhost)?),
    }
}

pub fn list_exchanges(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<Vec<responses::ExchangeInfo>> {
    let pagination = extract_pagination_params(command_args);
    match pagination {
        Some(params) => Ok(client.list_exchanges_in_paged(vhost, &params)?),
        None => Ok(client.list_exchanges_in(vhost)?),
    }
}

pub fn list_bindings(client: APIClient) -> CommandResult<Vec<responses::BindingInfo>> {
    Ok(client.list_bindings()?)
}

pub fn list_permissions(client: APIClient) -> CommandResult<Vec<responses::Permissions>> {
    Ok(client.list_permissions()?)
}

pub fn list_all_parameters(client: APIClient) -> CommandResult<Vec<responses::RuntimeParameter>> {
    Ok(client.list_runtime_parameters()?)
}

pub fn list_parameters(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<Vec<responses::RuntimeParameter>> {
    match command_args.get_one::<String>("component") {
        None => {
            let mut r = client.list_runtime_parameters()?;
            r.retain(|p| p.vhost == vhost);
            Ok(r)
        }
        Some(c) => Ok(client.list_runtime_parameters_of_component_in(c, vhost)?),
    }
}

pub fn list_parameters_of_component_in(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<Vec<responses::RuntimeParameter>> {
    let component = command_args.str_arg("component");
    Ok(client.list_runtime_parameters_of_component_in(component, vhost)?)
}

pub fn list_global_parameters(
    client: APIClient,
) -> CommandResult<Vec<responses::GlobalRuntimeParameter>> {
    Ok(client.list_global_runtime_parameters()?)
}

pub fn list_feature_flags(client: APIClient) -> CommandResult<responses::FeatureFlagList> {
    Ok(client.list_feature_flags()?)
}

pub fn list_shovels(client: APIClient) -> CommandResult<Vec<responses::Shovel>> {
    Ok(client.list_shovels()?)
}

pub fn list_shovels_in(client: APIClient, vhost: &str) -> CommandResult<Vec<responses::Shovel>> {
    Ok(client.list_shovels_in(vhost)?)
}

pub fn declare_amqp10_shovel(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let name = command_args.string_arg("name");
    let source_uri = command_args.string_arg("source_uri");
    let destination_uri = command_args.string_arg("destination_uri");

    let source_address = command_args.string_arg("source_address");
    let destination_address = command_args.string_arg("destination_address");

    let ack_mode = command_args
        .get_one::<MessageTransferAcknowledgementMode>("ack_mode")
        .cloned()
        .unwrap();
    let reconnect_delay = command_args
        .optional_typed::<u32>("reconnect_delay")
        .or(Some(5));

    let source_params = Amqp10ShovelSourceParams {
        source_address: &source_address,
        source_uri: &source_uri,
    };

    let destination_params = Amqp10ShovelDestinationParams {
        destination_address: &destination_address,
        destination_uri: &destination_uri,
    };

    let params = Amqp10ShovelParams {
        name: &name,
        vhost,
        source: source_params,
        destination: destination_params,
        acknowledgement_mode: ack_mode,
        reconnect_delay,
    };

    Ok(client.declare_amqp10_shovel(params)?)
}

pub fn declare_amqp091_shovel(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let name = command_args.string_arg("name");
    let source_uri = command_args.string_arg("source_uri");
    let destination_uri = command_args.string_arg("destination_uri");

    let ack_mode = command_args
        .get_one::<MessageTransferAcknowledgementMode>("ack_mode")
        .cloned()
        .unwrap();
    let reconnect_delay = command_args
        .optional_typed::<u32>("reconnect_delay")
        .or(Some(5));

    let predeclared_source = command_args.optional_typed_or::<bool>("predeclared_source", false);
    let source_queue_opt = command_args.optional_string("source_queue");
    let source_exchange_opt = command_args.optional_string("source_exchange");
    let source_exchange_routing_key_opt = command_args
        .get_one::<String>("source_exchange_key")
        .map(|s| s.as_str());

    let predeclared_destination =
        command_args.optional_typed_or::<bool>("predeclared_destination", false);
    let destination_queue_opt = command_args.optional_string("destination_queue");
    let destination_exchange_opt = command_args.optional_string("destination_exchange");
    let destination_exchange_routing_key_opt = command_args
        .get_one::<String>("destination_exchange_key")
        .map(|s| s.as_str());

    // Variables declared here to ensure they outlive the params structs which hold references
    let (source_queue, source_exchange);
    #[allow(clippy::unnecessary_unwrap)]
    let source_params = if source_queue_opt.is_some() {
        source_queue = source_queue_opt.expect("checked above");
        if predeclared_source {
            Amqp091ShovelSourceParams::predeclared_queue_source(&source_uri, &source_queue)
        } else {
            Amqp091ShovelSourceParams::queue_source(&source_uri, &source_queue)
        }
    } else {
        source_exchange = source_exchange_opt
            .expect("clap ensures that either source_queue or source_exchange is provided");
        if predeclared_source {
            Amqp091ShovelSourceParams::predeclared_exchange_source(
                &source_uri,
                &source_exchange,
                source_exchange_routing_key_opt,
            )
        } else {
            Amqp091ShovelSourceParams::exchange_source(
                &source_uri,
                &source_exchange,
                source_exchange_routing_key_opt,
            )
        }
    };

    let (destination_queue, destination_exchange);
    #[allow(clippy::unnecessary_unwrap)]
    let destination_params = if destination_queue_opt.is_some() {
        destination_queue = destination_queue_opt.expect("checked above");
        if predeclared_destination {
            Amqp091ShovelDestinationParams::predeclared_queue_destination(
                &destination_uri,
                &destination_queue,
            )
        } else {
            Amqp091ShovelDestinationParams::queue_destination(&destination_uri, &destination_queue)
        }
    } else {
        destination_exchange = destination_exchange_opt.expect(
            "clap ensures that either destination_queue or destination_exchange is provided",
        );
        if predeclared_destination {
            Amqp091ShovelDestinationParams::predeclared_exchange_destination(
                &destination_uri,
                &destination_exchange,
                destination_exchange_routing_key_opt,
            )
        } else {
            Amqp091ShovelDestinationParams::exchange_destination(
                &destination_uri,
                &destination_exchange,
                destination_exchange_routing_key_opt,
            )
        }
    };

    let params = Amqp091ShovelParams {
        name: &name,
        vhost,
        acknowledgement_mode: ack_mode,
        reconnect_delay,
        source: source_params,
        destination: destination_params,
    };
    Ok(client.declare_amqp091_shovel(params)?)
}

pub fn delete_shovel(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let name = command_args.string_arg("name");
    let idempotently = command_args.optional_typed_or::<bool>("idempotently", false);

    Ok(client.delete_shovel(vhost, &name, idempotently)?)
}

//
// Federation
//

pub fn list_federation_upstreams(
    client: APIClient,
) -> CommandResult<Vec<responses::FederationUpstream>> {
    Ok(client.list_federation_upstreams()?)
}

pub fn list_federation_links(client: APIClient) -> CommandResult<Vec<responses::FederationLink>> {
    Ok(client.list_federation_links()?)
}

struct FederationUpstreamCoreSettings {
    name: String,
    uri: String,
    reconnect_delay: u32,
    trust_user_id: bool,
    prefetch_count: u32,
    ack_mode: MessageTransferAcknowledgementMode,
    bind_using_nowait: bool,
    channel_use_mode: ChannelUseMode,
}

fn extract_federation_core_settings(command_args: &ArgMatches) -> FederationUpstreamCoreSettings {
    FederationUpstreamCoreSettings {
        name: command_args.string_arg("name"),
        uri: command_args.string_arg("uri"),
        reconnect_delay: command_args
            .get_one::<u32>("reconnect_delay")
            .cloned()
            .unwrap(),
        trust_user_id: command_args
            .get_one::<bool>("trust_user_id")
            .cloned()
            .unwrap(),
        prefetch_count: command_args
            .get_one::<u32>("prefetch_count")
            .cloned()
            .unwrap(),
        ack_mode: command_args
            .get_one::<MessageTransferAcknowledgementMode>("ack_mode")
            .cloned()
            .unwrap(),
        bind_using_nowait: command_args.optional_typed_or::<bool>("bind_nowait", false),
        channel_use_mode: command_args
            .get_one::<ChannelUseMode>("channel_use_mode")
            .cloned()
            .unwrap_or_default(),
    }
}

fn extract_exchange_federation_params(command_args: &ArgMatches) -> ExchangeFederationParams<'_> {
    let exchange_name = command_args
        .get_one::<String>("exchange_name")
        .map(|s| s.as_str());
    let queue_type = command_args
        .optional_string("queue_type")
        .map(|s| Into::<QueueType>::into(s.as_str()))
        .unwrap_or_default();
    let max_hops = command_args.get_one::<u8>("max_hops").copied();
    let resource_cleanup_mode = command_args
        .get_one::<FederationResourceCleanupMode>("resource_cleanup_mode")
        .cloned()
        .unwrap_or_default();
    let ttl = command_args.optional_typed::<u32>("ttl");
    let message_ttl = command_args.optional_typed::<u32>("message_ttl");

    ExchangeFederationParams {
        exchange: exchange_name,
        max_hops,
        queue_type,
        ttl,
        message_ttl,
        resource_cleanup_mode,
    }
}

pub fn declare_federation_upstream(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let core = extract_federation_core_settings(command_args);
    let efp = Some(extract_exchange_federation_params(command_args));

    let queue_name = command_args.optional_string("queue_name");
    let consumer_tag = command_args.optional_string("consumer_tag");
    let qfp = match (&queue_name, &consumer_tag) {
        (Some(qn), Some(ct)) => Some(QueueFederationParams::new_with_consumer_tag(qn, ct)),
        (Some(qn), None) => Some(QueueFederationParams::new(qn)),
        _ => None,
    };

    let upstream = FederationUpstreamParams {
        name: &core.name,
        vhost,
        uri: &core.uri,
        reconnect_delay: core.reconnect_delay,
        trust_user_id: core.trust_user_id,
        prefetch_count: core.prefetch_count,
        ack_mode: core.ack_mode,
        bind_using_nowait: core.bind_using_nowait,
        channel_use_mode: core.channel_use_mode,
        queue_federation: qfp,
        exchange_federation: efp,
    };
    let param = RuntimeParameterDefinition::from(upstream);
    Ok(client.upsert_runtime_parameter(&param)?)
}

pub fn declare_federation_upstream_for_exchange_federation(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let core = extract_federation_core_settings(command_args);
    let efp = Some(extract_exchange_federation_params(command_args));

    let upstream = FederationUpstreamParams {
        name: &core.name,
        vhost,
        uri: &core.uri,
        reconnect_delay: core.reconnect_delay,
        trust_user_id: core.trust_user_id,
        prefetch_count: core.prefetch_count,
        ack_mode: core.ack_mode,
        bind_using_nowait: core.bind_using_nowait,
        channel_use_mode: core.channel_use_mode,
        queue_federation: None,
        exchange_federation: efp,
    };
    let param = RuntimeParameterDefinition::from(upstream);
    Ok(client.upsert_runtime_parameter(&param)?)
}

pub fn declare_federation_upstream_for_queue_federation(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let core = extract_federation_core_settings(command_args);

    let queue_name = command_args.optional_string("queue_name");
    let consumer_tag = command_args.optional_string("consumer_tag");
    let qfp = match (&queue_name, &consumer_tag) {
        (Some(qn), Some(ct)) => Some(QueueFederationParams::new_with_consumer_tag(qn, ct)),
        (Some(qn), None) => Some(QueueFederationParams::new(qn)),
        _ => None,
    };

    let upstream = FederationUpstreamParams {
        name: &core.name,
        vhost,
        uri: &core.uri,
        reconnect_delay: core.reconnect_delay,
        trust_user_id: core.trust_user_id,
        prefetch_count: core.prefetch_count,
        ack_mode: core.ack_mode,
        bind_using_nowait: core.bind_using_nowait,
        channel_use_mode: core.channel_use_mode,
        queue_federation: qfp,
        exchange_federation: None,
    };
    let param = RuntimeParameterDefinition::from(upstream);
    Ok(client.upsert_runtime_parameter(&param)?)
}

pub fn delete_federation_upstream(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let name = command_args.string_arg("name");
    let idempotently = command_args.optional_typed_or::<bool>("idempotently", false);
    Ok(
        client.clear_runtime_parameter(
            FEDERATION_UPSTREAM_COMPONENT,
            vhost,
            &name,
            idempotently,
        )?,
    )
}

pub fn disable_tls_peer_verification_for_all_federation_upstreams(
    client: APIClient,
    prog_rep: &mut dyn ProgressReporter,
) -> Result<(), CommandRunError> {
    update_all_federation_upstream_uris(&client, prog_rep, disable_tls_peer_verification)
}

pub fn enable_tls_peer_verification_for_all_federation_upstreams(
    client: APIClient,
    args: &ArgMatches,
    prog_rep: &mut dyn ProgressReporter,
) -> Result<(), CommandRunError> {
    let cert_paths = CertPaths::from_args(args)?;
    update_all_federation_upstream_uris(&client, prog_rep, |uri| {
        enable_tls_peer_verification(
            uri,
            cert_paths.ca_cert_path,
            cert_paths.client_cert_path,
            cert_paths.client_key_path,
        )
    })
}

pub fn disable_tls_peer_verification_for_all_source_uris(
    client: APIClient,
    prog_rep: &mut dyn ProgressReporter,
) -> Result<(), CommandRunError> {
    update_all_shovel_uris(
        &client,
        prog_rep,
        ShovelUriField::Source,
        disable_tls_peer_verification,
    )
}

pub fn disable_tls_peer_verification_for_all_destination_uris(
    client: APIClient,
    prog_rep: &mut dyn ProgressReporter,
) -> Result<(), CommandRunError> {
    update_all_shovel_uris(
        &client,
        prog_rep,
        ShovelUriField::Destination,
        disable_tls_peer_verification,
    )
}

pub fn enable_tls_peer_verification_for_all_source_uris(
    client: APIClient,
    args: &ArgMatches,
    prog_rep: &mut dyn ProgressReporter,
) -> Result<(), CommandRunError> {
    let cert_paths = CertPaths::from_args(args)?;
    update_all_shovel_uris(&client, prog_rep, ShovelUriField::Source, |uri| {
        enable_tls_peer_verification(
            uri,
            cert_paths.ca_cert_path,
            cert_paths.client_cert_path,
            cert_paths.client_key_path,
        )
    })
}

pub fn enable_tls_peer_verification_for_all_destination_uris(
    client: APIClient,
    args: &ArgMatches,
    prog_rep: &mut dyn ProgressReporter,
) -> Result<(), CommandRunError> {
    let cert_paths = CertPaths::from_args(args)?;
    update_all_shovel_uris(&client, prog_rep, ShovelUriField::Destination, |uri| {
        enable_tls_peer_verification(
            uri,
            cert_paths.ca_cert_path,
            cert_paths.client_cert_path,
            cert_paths.client_key_path,
        )
    })
}

//
// Feature flags
//

pub fn enable_feature_flag(client: APIClient, command_args: &ArgMatches) -> CommandResult<()> {
    let name = command_args.string_arg("name");
    Ok(client.enable_feature_flag(&name)?)
}

pub fn enable_all_stable_feature_flags(client: APIClient) -> CommandResult<()> {
    Ok(client.enable_all_stable_feature_flags()?)
}

//
// Deprecated features
//

pub fn list_deprecated_features(
    client: APIClient,
) -> CommandResult<responses::DeprecatedFeatureList> {
    Ok(client.list_all_deprecated_features()?)
}

pub fn list_deprecated_features_in_use(
    client: APIClient,
) -> CommandResult<responses::DeprecatedFeatureList> {
    Ok(client.list_deprecated_features_in_use()?)
}

//
// Plugins
//

#[derive(Debug, Clone, Tabled)]
pub struct PluginOnNode {
    pub node: String,
    pub name: String,
    pub state: String,
}

pub fn list_plugins_on_node(
    client: APIClient,
    command_args: &ArgMatches,
) -> CommandResult<Vec<PluginOnNode>> {
    let node = command_args.string_arg("node");
    let plugins = client.list_node_plugins(&node)?;

    Ok(plugins
        .into_iter()
        .map(|plugin_name| PluginOnNode {
            node: node.clone(),
            name: plugin_name,
            state: "Enabled".to_string(),
        })
        .collect())
}

pub fn list_plugins_across_cluster(client: APIClient) -> CommandResult<Vec<PluginOnNode>> {
    let nodes = client.list_nodes()?;
    let mut result = Vec::new();

    for node in nodes {
        let plugins = client.list_node_plugins(&node.name)?;
        for plugin_name in plugins {
            result.push(PluginOnNode {
                node: node.name.clone(),
                name: plugin_name,
                state: "Enabled".to_string(),
            });
        }
    }

    Ok(result)
}

//
// Declaration of core resources
//

pub fn declare_vhost(client: APIClient, command_args: &ArgMatches) -> CommandResult<()> {
    let name = command_args.str_arg("name");
    let description = command_args
        .get_one::<String>("description")
        .map(|s| s.as_str());
    let dqt = command_args
        .optional_string("default_queue_type")
        .map(|s| Into::<QueueType>::into(s.as_str()));
    // TODO: tags
    let tracing = command_args.optional_typed_or::<bool>("tracing", false);

    let params = requests::VirtualHostParams {
        name,
        description,
        default_queue_type: dqt,
        tags: None,
        tracing,
    };

    Ok(client.create_vhost(&params)?)
}

pub fn declare_exchange(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.str_arg("name");
    let exchange_type = command_args
        .get_one::<ExchangeType>("type")
        .cloned()
        .unwrap_or(commons::ExchangeType::Direct);
    let durable = command_args.optional_typed_or::<bool>("durable", true);
    let auto_delete = command_args.optional_typed_or::<bool>("auto_delete", false);
    let arguments = command_args.str_arg("arguments");

    let params = requests::ExchangeParams {
        name,
        exchange_type,
        durable,
        auto_delete,
        arguments: parse_json_from_arg(arguments)?,
    };

    client.declare_exchange(vhost, &params).map_err(Into::into)
}

pub fn declare_binding(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let source = command_args.str_arg("source");
    let destination_type = command_args
        .get_one::<BindingDestinationType>("destination_type")
        .unwrap();
    let destination = command_args.str_arg("destination");
    let routing_key = command_args.str_arg("routing_key");
    let arguments = command_args.str_arg("arguments");
    let parsed_arguments = parse_json_from_arg(arguments)?;

    match destination_type {
        BindingDestinationType::Queue => client
            .bind_queue(
                vhost,
                destination,
                source,
                Some(routing_key),
                parsed_arguments,
            )
            .map_err(Into::into),
        BindingDestinationType::Exchange => client
            .bind_exchange(
                vhost,
                destination,
                source,
                Some(routing_key),
                parsed_arguments,
            )
            .map_err(Into::into),
    }
}

pub fn declare_vhost_limit(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.str_arg("name");
    let value = command_args.str_arg("value");

    let parsed_value = str::parse(value).map_err(|_| CommandRunError::JsonParseError {
        message: format!("'{}' is not a valid integer value", value),
    })?;

    let limit = EnforcedLimitParams::new(VirtualHostLimitTarget::from(name.as_str()), parsed_value);

    client.set_vhost_limit(vhost, limit).map_err(Into::into)
}

pub fn declare_user_limit(
    client: APIClient,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let user = command_args.str_arg("user");
    let name = command_args.str_arg("name");
    let value = command_args.str_arg("value");

    let parsed_value = str::parse(value).map_err(|_| CommandRunError::JsonParseError {
        message: format!("'{}' is not a valid integer value", value),
    })?;

    let limit = EnforcedLimitParams::new(UserLimitTarget::from(name.as_str()), parsed_value);

    client.set_user_limit(user, limit).map_err(Into::into)
}

pub fn delete_vhost_limit(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let name = command_args.str_arg("name");

    Ok(client.clear_vhost_limit(vhost, VirtualHostLimitTarget::from(name.as_str()))?)
}

pub fn delete_user_limit(client: APIClient, command_args: &ArgMatches) -> CommandResult<()> {
    let user = command_args.str_arg("user");
    let name = command_args.str_arg("name");

    Ok(client.clear_user_limit(user, UserLimitTarget::from(name.as_str()))?)
}

pub fn delete_parameter(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let component = command_args.str_arg("component");
    let name = command_args.str_arg("name");
    let idempotently = command_args.optional_typed_or::<bool>("idempotently", false);

    Ok(client.clear_runtime_parameter(component, vhost, name, idempotently)?)
}

pub fn delete_global_parameter(client: APIClient, command_args: &ArgMatches) -> CommandResult<()> {
    let name = command_args.str_arg("name");

    Ok(client.clear_global_runtime_parameter(name)?)
}

pub fn delete_vhost(client: APIClient, command_args: &ArgMatches) -> CommandResult<()> {
    let name = command_args.str_arg("name");
    let idempotently = command_args.optional_typed_or::<bool>("idempotently", false);
    Ok(client.delete_vhost(name, idempotently)?)
}

pub fn enable_vhost_deletion_protection(
    client: APIClient,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let name = command_args.str_arg("name");
    Ok(client.enable_vhost_deletion_protection(name)?)
}

pub fn disable_vhost_deletion_protection(
    client: APIClient,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let name = command_args.str_arg("name");
    Ok(client.disable_vhost_deletion_protection(name)?)
}

pub fn delete_multiple_vhosts(
    client: APIClient,
    command_args: &ArgMatches,
    prog_rep: &mut dyn ProgressReporter,
) -> Result<Option<Vec<responses::VirtualHost>>, CommandRunError> {
    let name_pattern = command_args.str_arg("name_pattern");
    let approve = command_args.optional_typed_or::<bool>("approve", false);
    let dry_run = command_args.optional_typed_or::<bool>("dry_run", false);
    let idempotently = command_args.optional_typed_or::<bool>("idempotently", false);
    let non_interactive_cli = command_args
        .optional_typed::<bool>("non_interactive")
        .unwrap_or_else(|| pre_flight::InteractivityMode::from_env().is_non_interactive());

    let regex =
        Regex::new(name_pattern).map_err(|_| CommandRunError::UnsupportedArgumentValue {
            property: "name_pattern".to_string(),
        })?;

    let vhosts = client.list_vhosts()?;

    let matching_vhosts: Vec<responses::VirtualHost> = vhosts
        .into_iter()
        .filter(|vhost| regex.is_match(&vhost.name))
        .filter(|vhost| vhost.name != DEFAULT_VHOST)
        .collect();

    if dry_run {
        return Ok(Some(matching_vhosts));
    }

    if !approve && !pre_flight::is_non_interactive() && !non_interactive_cli {
        return Err(CommandRunError::FailureDuringExecution {
            message: "This operation is destructive and requires the --approve flag".to_string(),
        });
    }

    let total = matching_vhosts.len();

    if total == 0 {
        return Ok(None);
    }

    prog_rep.start_operation(total, "Deleting virtual hosts");

    let mut successes = 0;
    let mut failures = 0;

    for (index, vhost) in matching_vhosts.iter().enumerate() {
        let vhost_name = &vhost.name;
        match client.delete_vhost(vhost_name, idempotently) {
            Ok(_) => {
                prog_rep.report_progress(index + 1, total, vhost_name);
                successes += 1;
            }
            Err(error) => {
                prog_rep.report_failure(vhost_name, &error.to_string());
                failures += 1;
            }
        }
    }

    prog_rep.finish_operation(total);

    if failures > 0 && successes == 0 {
        return Err(CommandRunError::FailureDuringExecution {
            message: format!("Failed to delete all {} virtual hosts", failures),
        });
    }

    Ok(None)
}

pub fn delete_user(client: APIClient, command_args: &ArgMatches) -> CommandResult<()> {
    let name = command_args.str_arg("name");
    let idempotently = command_args.optional_typed_or::<bool>("idempotently", false);
    Ok(client.delete_user(name, idempotently)?)
}

pub fn delete_permissions(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let user = command_args.str_arg("user");
    let idempotently = command_args.optional_typed_or::<bool>("idempotently", false);
    Ok(client.clear_permissions(vhost, user, idempotently)?)
}

pub fn declare_user(client: APIClient, command_args: &ArgMatches) -> CommandResult<()> {
    let name = command_args.str_arg("name");
    let password = command_args.str_arg("password");
    let provided_hash = command_args.str_arg("password_hash");
    let tags = command_args.str_arg("tags");

    let has_password = !password.is_empty();
    let has_hash = !provided_hash.is_empty();

    if !has_password && !has_hash {
        return Err(CommandRunError::MissingOptions {
            message: "Please provide either --password or --password-hash".to_string(),
        });
    }

    if has_password && has_hash {
        return Err(CommandRunError::ConflictingOptions {
            message: "Please provide either --password or --password-hash, not both".to_string(),
        });
    }

    let password_hash = if provided_hash.is_empty() {
        let hashing_algo = command_args
            .get_one::<HashingAlgorithm>("hashing_algorithm")
            .unwrap();
        let salt = password_hashing::salt();
        hashing_algo.salt_and_hash(&salt, password).map_err(|e| {
            CommandRunError::FailureDuringExecution {
                message: format!("Password hashing failed: {}", e),
            }
        })?
    } else {
        provided_hash.to_owned()
    };

    let params = requests::UserParams {
        name,
        password_hash: password_hash.as_str(),
        tags,
    };
    Ok(client.create_user(&params)?)
}

pub fn salt_and_hash_password(command_args: &ArgMatches) -> Result<String, HashingError> {
    let password = command_args.string_arg("password");
    let hashing_algo = command_args
        .get_one::<HashingAlgorithm>("hashing_algorithm")
        .unwrap();

    let salt = password_hashing::salt();
    let password_hash = hashing_algo.salt_and_hash(&salt, &password)?;

    Ok(password_hash)
}

pub fn declare_permissions(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let user = command_args.str_arg("user");
    let configure = command_args.str_arg("configure");
    let read = command_args.str_arg("read");
    let write = command_args.str_arg("write");

    let params = requests::Permissions {
        user,
        vhost,
        configure,
        read,
        write,
    };

    Ok(client.declare_permissions(&params)?)
}

pub fn declare_queue(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.str_arg("name");
    let queue_type = command_args.get_one::<QueueType>("type").cloned().unwrap();

    let durable = command_args.optional_typed_or::<bool>("durable", true);
    let auto_delete = command_args.optional_typed_or::<bool>("auto_delete", false);
    let arguments = command_args.str_arg("arguments");
    let parsed_args = parse_json_from_arg(arguments)?;

    let params = requests::QueueParams::new(name, queue_type, durable, auto_delete, parsed_args);

    client.declare_queue(vhost, &params).map_err(Into::into)
}

pub fn declare_stream(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.str_arg("name");
    let expiration = command_args.str_arg("expiration");
    let max_length_bytes = command_args.optional_typed::<u64>("max_length_bytes");
    let max_segment_length_bytes = command_args.optional_typed::<u64>("max_segment_length_bytes");
    let arguments = command_args.str_arg("arguments");
    let parsed_args = parse_json_from_arg(arguments)?;

    let params = requests::StreamParams {
        name,
        expiration,
        max_length_bytes,
        max_segment_length_bytes,
        arguments: parsed_args,
    };

    client.declare_stream(vhost, &params).map_err(Into::into)
}

pub fn declare_policy(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.str_arg("name");
    let pattern = command_args.str_arg("pattern");
    let apply_to = command_args
        .get_one::<PolicyTarget>("apply_to")
        .cloned()
        .unwrap();
    let priority: i32 = command_args.parse_required("priority")?;
    let definition = command_args.str_arg("definition");

    let parsed_definition = parse_json_from_arg(definition)?;

    let params = PolicyParams {
        vhost,
        name,
        pattern,
        apply_to,
        priority,
        definition: parsed_definition,
    };

    client.declare_policy(&params).map_err(Into::into)
}

pub fn declare_operator_policy(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.string_arg("name");
    let pattern = command_args.string_arg("pattern");
    let apply_to = command_args
        .get_one::<PolicyTarget>("apply_to")
        .cloned()
        .unwrap();
    let priority: i32 = command_args.parse_required("priority")?;
    let definition = command_args.str_arg("definition");

    let parsed_definition = parse_json_from_arg(definition)?;

    let params = PolicyParams {
        vhost,
        name: &name,
        pattern: &pattern,
        apply_to,
        priority,
        definition: parsed_definition,
    };

    client.declare_operator_policy(&params).map_err(Into::into)
}

pub fn declare_policy_override(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let original_pol_name = command_args.string_arg("name");
    let override_pol_name = command_args
        .optional_string("override_name")
        .unwrap_or_else(|| override_policy_name(&original_pol_name));

    let existing_policy = client
        .get_policy(vhost, &original_pol_name)
        .map_err(CommandRunError::from)?;

    let new_priority = existing_policy.priority + 100;
    let definition = command_args.str_arg("definition");

    let parsed_definition = parse_json_from_arg(definition)?;

    let overridden =
        existing_policy.with_overrides(&override_pol_name, new_priority, &parsed_definition);
    let params = PolicyParams::from(&overridden);
    client.declare_policy(&params).map_err(Into::into)
}

pub fn declare_blanket_policy(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let existing_policies = client
        .list_policies_in(vhost)
        .map_err(CommandRunError::from)?;
    let min_priority = existing_policies
        .iter()
        .map(|p| p.priority)
        .min()
        .unwrap_or(0);

    // blanket policy priority should be the lowest in the virtual host
    let priority = [min_priority - 1, DEFAULT_BLANKET_POLICY_PRIORITY]
        .iter()
        .min()
        .cloned()
        .unwrap();

    let name = command_args.string_arg("name");
    let apply_to = command_args
        .get_one::<PolicyTarget>("apply_to")
        .cloned()
        .unwrap();
    let definition = command_args.str_arg("definition");

    let parsed_definition = parse_json_from_arg(definition)?;

    let params = PolicyParams {
        vhost,
        name: &name,
        pattern: ".*",
        apply_to,
        priority: priority as i32,
        definition: parsed_definition,
    };

    client.declare_policy(&params).map_err(Into::into)
}

pub fn update_policy_definition(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.string_arg("name");
    let key = command_args.string_arg("definition_key");
    let value = command_args.string_arg("definition_value");
    let parsed_value = parse_json_from_arg::<Value>(&value)?;

    update_policy_definition_with(&client, vhost, &name, &key, &parsed_value)
}

pub fn update_operator_policy_definition(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.string_arg("name");
    let key = command_args.string_arg("definition_key");
    let value = command_args.string_arg("definition_value");
    let parsed_value = parse_json_from_arg::<Value>(&value)?;

    update_operator_policy_definition_with(&client, vhost, &name, &key, &parsed_value)
}

pub fn patch_policy_definition(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.string_arg("name");
    let value = command_args.string_arg("definition");
    let parsed_value = parse_json_from_arg::<Value>(&value)?;

    let mut pol = client
        .get_policy(vhost, &name)
        .map_err(CommandRunError::from)?;
    let patch = parsed_value
        .as_object()
        .ok_or_else(|| CommandRunError::JsonParseError {
            message: "definition must be a JSON object".to_string(),
        })?;
    for (k, v) in patch.iter() {
        pol.insert_definition_key(k.clone(), v.clone());
    }

    let params = PolicyParams::from(&pol);
    client.declare_policy(&params).map_err(Into::into)
}

pub fn update_all_policy_definitions_in(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let pols = client
        .list_policies_in(vhost)
        .map_err(CommandRunError::from)?;
    let key = command_args.string_arg("definition_key");
    let value = command_args.string_arg("definition_value");
    let parsed_value = parse_json_from_arg::<Value>(&value)?;

    for pol in pols {
        update_policy_definition_with(&client, vhost, &pol.name, &key, &parsed_value)?
    }

    Ok(())
}

pub fn patch_operator_policy_definition(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.string_arg("name");
    let value = command_args.string_arg("definition");
    let parsed_value = parse_json_from_arg::<Value>(&value)?;

    let mut pol = client
        .get_operator_policy(vhost, &name)
        .map_err(CommandRunError::from)?;
    let patch = parsed_value
        .as_object()
        .ok_or_else(|| CommandRunError::JsonParseError {
            message: "definition must be a JSON object".to_string(),
        })?;
    for (k, v) in patch.iter() {
        pol.insert_definition_key(k.clone(), v.clone());
    }

    let params = PolicyParams::from(&pol);
    client.declare_operator_policy(&params).map_err(Into::into)
}

pub fn update_all_operator_policy_definitions_in(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let pols = client
        .list_operator_policies_in(vhost)
        .map_err(CommandRunError::from)?;
    let key = command_args.string_arg("definition_key");
    let value = command_args.string_arg("definition_value");
    let parsed_value = parse_json_from_arg::<Value>(&value)?;

    for pol in pols {
        update_operator_policy_definition_with(&client, vhost, &pol.name, &key, &parsed_value)?
    }

    Ok(())
}

pub fn delete_policy_definition_keys(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let name = command_args.string_arg("name");
    let keys: Vec<&str> = command_args
        .get_many::<String>("definition_keys")
        .unwrap()
        .map(String::as_str)
        .collect();

    let pol = client.get_policy(vhost, &name)?;
    let updated_pol = pol.without_keys(&keys);

    let params = PolicyParams::from(&updated_pol);
    Ok(client.declare_policy(&params)?)
}

pub fn delete_policy_definition_keys_in(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let pols = client.list_policies_in(vhost)?;
    let keys: Vec<&str> = command_args
        .get_many::<String>("definition_keys")
        .unwrap()
        .map(String::as_str)
        .collect();

    for pol in pols {
        let updated_pol = pol.without_keys(&keys);

        let params = PolicyParams::from(&updated_pol);
        client.declare_policy(&params)?
    }

    Ok(())
}

pub fn delete_operator_policy_definition_keys(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let name = command_args.string_arg("name");
    let keys: Vec<&str> = command_args
        .get_many::<String>("definition_keys")
        .unwrap()
        .map(String::as_str)
        .collect();

    let pol = client.get_operator_policy(vhost, &name)?;
    let updated_pol = pol.without_keys(&keys);

    let params = PolicyParams::from(&updated_pol);
    Ok(client.declare_operator_policy(&params)?)
}

pub fn delete_operator_policy_definition_keys_in(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let pols = client.list_operator_policies_in(vhost)?;
    let keys: Vec<&str> = command_args
        .get_many::<String>("definition_keys")
        .unwrap()
        .map(String::as_str)
        .collect();

    for pol in pols {
        let updated_pol = pol.without_keys(&keys);

        let params = PolicyParams::from(&updated_pol);
        client.declare_operator_policy(&params)?
    }

    Ok(())
}

fn update_policy_definition_with(
    client: &APIClient,
    vhost: &str,
    name: &str,
    key: &str,
    parsed_value: &Value,
) -> CommandResult<()> {
    let mut policy = client.get_policy(vhost, name)?;
    policy.insert_definition_key(key.to_owned(), parsed_value.clone());

    let params = PolicyParams::from(&policy);
    Ok(client.declare_policy(&params)?)
}

fn update_operator_policy_definition_with(
    client: &APIClient,
    vhost: &str,
    name: &str,
    key: &str,
    parsed_value: &Value,
) -> CommandResult<()> {
    let mut policy = client.get_operator_policy(vhost, name)?;
    policy.insert_definition_key(key.to_owned(), parsed_value.clone());

    let params = PolicyParams::from(&policy);
    Ok(client.declare_operator_policy(&params)?)
}

pub fn declare_parameter(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let component = command_args.str_arg("component");
    let name = command_args.str_arg("name");
    let value = command_args.str_arg("value");
    let parsed_value = parse_json_from_arg(value)?;

    let params = requests::RuntimeParameterDefinition {
        vhost,
        name,
        component,
        value: parsed_value,
    };

    client.upsert_runtime_parameter(&params).map_err(Into::into)
}

pub fn declare_global_parameter(
    client: APIClient,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.str_arg("name");
    let value = command_args.str_arg("value");
    // TODO: global runtime parameter values can be regular strings (not JSON documents)
    //       but we don't support that yet in the HTTP API client.
    let parsed_value = parse_json_from_arg(value)?;

    let params = requests::GlobalRuntimeParameterDefinition {
        name,
        value: parsed_value,
    };

    client
        .upsert_global_runtime_parameter(&params)
        .map_err(Into::into)
}

pub fn delete_queue(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let name = command_args.str_arg("name");
    let idempotently = command_args.optional_typed_or::<bool>("idempotently", false);
    Ok(client.delete_queue(vhost, name, idempotently)?)
}

pub fn delete_stream(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    delete_queue(client, vhost, command_args)
}

pub fn delete_binding(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let source = command_args.str_arg("source");
    let destination_type = command_args.string_arg("destination_type");
    let destination = command_args.str_arg("destination");
    let routing_key = command_args.str_arg("routing_key");
    let arguments = command_args.str_arg("arguments");
    let parsed_arguments = parse_json_from_arg(arguments)?;

    let params = BindingDeletionParams {
        virtual_host: vhost,
        source,
        destination,
        destination_type: BindingDestinationType::from(destination_type),
        routing_key,
        arguments: parsed_arguments,
    };
    let idempotently = command_args.optional_typed_or::<bool>("idempotently", false);

    client
        .delete_binding(&params, idempotently)
        .map(|_| ())
        .map_err(Into::into)
}

pub fn delete_exchange(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let name = command_args.str_arg("name");
    let idempotent = command_args.optional_typed_or::<bool>("idempotently", false);
    Ok(client.delete_exchange(vhost, name, idempotent)?)
}

pub fn delete_policy(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let name = command_args.str_arg("name");
    let idempotently = command_args.optional_typed_or::<bool>("idempotently", false);
    Ok(client.delete_policy(vhost, name, idempotently)?)
}

pub fn delete_operator_policy(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let name = command_args.str_arg("name");
    let idempotently = command_args.optional_typed_or::<bool>("idempotently", false);
    Ok(client.delete_operator_policy(vhost, name, idempotently)?)
}

pub fn purge_queue(client: APIClient, vhost: &str, command_args: &ArgMatches) -> CommandResult<()> {
    let name = command_args.str_arg("name");
    Ok(client.purge_queue(vhost, name)?)
}

pub fn health_check_local_alarms(client: APIClient) -> CommandResult<()> {
    Ok(client.health_check_local_alarms()?)
}

pub fn health_check_cluster_wide_alarms(client: APIClient) -> CommandResult<()> {
    Ok(client.health_check_cluster_wide_alarms()?)
}

pub fn health_check_node_is_quorum_critical(client: APIClient) -> CommandResult<()> {
    Ok(client.health_check_if_node_is_quorum_critical()?)
}

pub fn health_check_port_listener(
    client: APIClient,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let port = command_args.get_one::<u16>("port").cloned().unwrap();
    Ok(client.health_check_port_listener(port)?)
}

pub fn health_check_protocol_listener(
    client: APIClient,
    command_args: &ArgMatches,
) -> CommandResult<()> {
    let proto = command_args
        .get_one::<SupportedProtocol>("protocol")
        .cloned()
        .unwrap();
    Ok(client.health_check_protocol_listener(proto)?)
}

pub fn close_connection(client: APIClient, command_args: &ArgMatches) -> CommandResult<()> {
    let name = command_args.str_arg("name");
    let idempotently = command_args.optional_typed_or::<bool>("idempotently", false);
    Ok(client.close_connection(name, Some("closed via rabbitmqadmin v2"), idempotently)?)
}

pub fn close_user_connections(client: APIClient, command_args: &ArgMatches) -> CommandResult<()> {
    let username = command_args.str_arg("username");
    let idempotently = command_args.optional_typed_or::<bool>("idempotently", false);
    Ok(client.close_user_connections(
        username,
        Some("closed via rabbitmqadmin v2"),
        idempotently,
    )?)
}

pub fn rebalance_queues(client: APIClient) -> CommandResult<()> {
    Ok(client.rebalance_queue_leaders()?)
}

pub fn export_cluster_wide_definitions(
    client: APIClient,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let transformations = command_args
        .get_many::<String>("transformations")
        .unwrap_or_default();

    if transformations.len() == 0 {
        export_cluster_wide_definitions_without_transformations(client, command_args)
    } else {
        let transformations = transformations.map(String::from).collect();

        export_and_transform_cluster_wide_definitions(client, command_args, transformations)
    }
}

fn export_and_transform_cluster_wide_definitions(
    client: APIClient,
    command_args: &ArgMatches,
    transformations: Vec<String>,
) -> Result<(), CommandRunError> {
    match client.export_cluster_wide_definitions_as_data() {
        Ok(mut defs0) => {
            let chain = TransformationChain::from(transformations);
            let defs1 = chain.apply(&mut defs0);
            let json = serde_json::to_string_pretty(&defs1).unwrap();

            let path = command_args.str_arg("file");
            match path.as_str() {
                "-" => {
                    println!("{}", &json);
                    Ok(())
                }
                file => {
                    fs::write(file, &json)?;
                    Ok(())
                }
            }
        }
        Err(err) => Err(err.into()),
    }
}

fn export_cluster_wide_definitions_without_transformations(
    client: APIClient,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    match client.export_cluster_wide_definitions() {
        Ok(definitions) => {
            let path = command_args.optional_string("file");
            let use_stdout = command_args.optional_typed::<bool>("stdout");
            match (path, use_stdout) {
                (Some(_val), Some(true)) => {
                    println!("{}", &definitions);
                    Ok(())
                }
                (Some(val), Some(false)) => match val.as_str() {
                    "-" => {
                        println!("{}", &definitions);
                        Ok(())
                    }
                    _ => {
                        fs::write(val, &definitions)?;
                        Ok(())
                    }
                },
                (_, Some(true)) => {
                    println!("{}", &definitions);
                    Ok(())
                }
                _ => Err(CommandRunError::MissingOptions {
                    message: "either --file or --stdout must be provided".to_string(),
                }),
            }
        }
        Err(err) => Err(err.into()),
    }
}

pub fn export_vhost_definitions(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let transformations = command_args
        .get_many::<String>("transformations")
        .unwrap_or_default();

    if transformations.len() == 0 {
        export_vhost_definitions_without_transformations(client, vhost, command_args)
    } else {
        let transformations = transformations.map(String::from).collect();

        export_and_transform_vhost_definitions(client, vhost, command_args, transformations)
    }
}

fn export_and_transform_vhost_definitions(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
    transformations: Vec<String>,
) -> Result<(), CommandRunError> {
    match client.export_vhost_definitions_as_data(vhost) {
        Ok(mut defs0) => {
            let chain = VirtualHostTransformationChain::from(transformations);
            chain.apply(&mut defs0);

            let json = serde_json::to_string_pretty(&defs0).unwrap();

            let path = command_args.str_arg("file");
            match path.as_str() {
                "-" => {
                    println!("{}", &json);
                    Ok(())
                }
                file => {
                    fs::write(file, &json)?;
                    Ok(())
                }
            }
        }
        Err(err) => Err(err.into()),
    }
}

fn export_vhost_definitions_without_transformations(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    match client.export_vhost_definitions(vhost) {
        Ok(definitions) => {
            let path = command_args.str_arg("file");
            match path.as_str() {
                "-" => {
                    println!("{}", &definitions);
                    Ok(())
                }
                file => {
                    fs::write(file, &definitions)?;
                    Ok(())
                }
            }
        }
        Err(err) => Err(err.into()),
    }
}

pub fn import_definitions(
    client: APIClient,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let defs_json = read_and_parse_definitions(command_args)?;
    client.import_definitions(defs_json).map_err(Into::into)
}

pub fn import_vhost_definitions(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let defs_json = read_and_parse_definitions(command_args)?;
    client
        .import_vhost_definitions(vhost, defs_json)
        .map_err(Into::into)
}

fn read_and_parse_definitions(command_args: &ArgMatches) -> Result<Value, CommandRunError> {
    let path = command_args.optional_string("file").map(|s| {
        s.trim_ascii()
            .trim_matches('\'')
            .trim_matches('"')
            .to_string()
    });
    let path_ref = path.as_deref();
    let use_stdin = command_args.optional_typed::<bool>("stdin");
    let definitions = read_definitions(path_ref, use_stdin).map_err(|err| {
        let message = match path_ref {
            None => format!("could not read from standard input: {}", err),
            Some(val) => format!("`{}` does not exist or is not readable: {}", val, err),
        };
        CommandRunError::FailureDuringExecution { message }
    })?;

    serde_json::from_str(definitions.as_str()).map_err(|err| {
        let message = match path_ref {
            None => format!("could not parse JSON from standard input: {}", err),
            Some(val) => format!("`{}` is not a valid JSON file: {}", val, err),
        };
        CommandRunError::FailureDuringExecution { message }
    })
}

const POLICY_LENGTH_LIMIT: usize = 255;
const OVERRIDE_POLICY_PREFIX: &str = "overrides.";

fn override_policy_name(original_policy_name: &str) -> String {
    let n = POLICY_LENGTH_LIMIT - OVERRIDE_POLICY_PREFIX.len();

    let mut s = original_policy_name.to_owned();
    s.truncate(n);

    format!("{}{}", OVERRIDE_POLICY_PREFIX, s)
}

fn read_definitions(path: Option<&str>, use_stdin: Option<bool>) -> io::Result<String> {
    match (path, use_stdin) {
        (_, Some(true)) => {
            let mut buffer = String::new();
            read_stdin_lines(&mut buffer)?;
            Ok(buffer)
        }
        (Some(val), _) => match val {
            "-" => {
                let mut buffer = String::new();
                read_stdin_lines(&mut buffer)?;
                Ok(buffer)
            }
            _ => fs::read_to_string(val),
        },
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "either an input file path or --stdin must be specified",
        )),
    }
}

fn read_stdin_lines(buffer: &mut String) -> io::Result<()> {
    let stdin = io::stdin();
    let lines = stdin.lines();
    for ln in lines {
        buffer.push_str(&ln?);
    }
    Ok(())
}

pub fn publish_message(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<responses::MessageRouted, CommandRunError> {
    let exchange = command_args.str_arg("exchange");
    let routing_key = command_args.str_arg("routing_key");
    let payload = command_args.str_arg("payload");
    let properties = command_args.str_arg("properties");
    let parsed_properties = parse_json_from_arg(properties)?;

    client
        .publish_message(vhost, exchange, routing_key, payload, parsed_properties)
        .map_err(Into::into)
}

pub fn get_messages(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> CommandResult<Vec<responses::GetMessage>> {
    let queue = command_args.str_arg("queue");
    let count: u32 = command_args.parse_required("count")?;
    let ack_mode = command_args.str_arg("ack_mode");
    Ok(client.get_messages(vhost, queue, count, ack_mode)?)
}

fn parse_json_from_arg<T: DeserializeOwned>(input: &str) -> Result<T, CommandRunError> {
    serde_json::from_str(input).map_err(|err| CommandRunError::JsonParseError {
        message: format!("`{}` is not a valid JSON: {}", input, err),
    })
}

pub fn disable_tls_peer_verification(uri: &str) -> Result<String, CommandRunError> {
    let ub = UriBuilder::new(uri)
        .map_err(|e| CommandRunError::FailureDuringExecution {
            message: format!("Could not parse a value as a URI '{}': {}", uri, e),
        })?
        .with_tls_peer_verification(TlsPeerVerificationMode::Disabled);

    ub.build()
        .map_err(|e| CommandRunError::FailureDuringExecution {
            message: format!("Failed to reconstruct (modify) a URI: {}", e),
        })
}

pub fn enable_tls_peer_verification(
    uri: &str,
    ca_cert_path: &str,
    client_cert_path: &str,
    client_key_path: &str,
) -> Result<String, CommandRunError> {
    let ub = UriBuilder::new(uri)
        .map_err(|e| CommandRunError::FailureDuringExecution {
            message: format!("Could not parse a value as a URI '{}': {}", uri, e),
        })?
        .with_tls_peer_verification(TlsPeerVerificationMode::Enabled)
        .with_ca_cert_file(ca_cert_path)
        .with_client_cert_file(client_cert_path)
        .with_client_key_file(client_key_path);

    ub.build()
        .map_err(|e| CommandRunError::FailureDuringExecution {
            message: format!("Failed to reconstruct (modify) a URI: {}", e),
        })
}

pub fn config_file_show_path(config_path: &Path) -> Result<Vec<ConfigPathEntry>, CommandRunError> {
    if !config_file_exists(config_path) {
        return Err(CommandRunError::FailureDuringExecution {
            message: format!(
                "Configuration file '{}' does not exist",
                config_path.display()
            ),
        });
    }

    let resolved = crate::config::resolve_config_file_path(Some(&config_path.to_path_buf()));

    Ok(vec![ConfigPathEntry {
        key: "Configuration file path".to_string(),
        value: resolved.to_string_lossy().to_string(),
    }])
}

pub fn config_file_show(
    config_path: &Path,
    reveal_passwords: bool,
) -> Result<Vec<NodeConfigEntry>, CommandRunError> {
    if !config_file_exists(config_path) {
        return Err(CommandRunError::FailureDuringExecution {
            message: format!(
                "Configuration file '{}' does not exist",
                config_path.display()
            ),
        });
    }

    match list_all_nodes(config_path) {
        Ok(nodes) => {
            let entries: Vec<NodeConfigEntry> = nodes
                .into_iter()
                .map(|(name, settings)| {
                    NodeConfigEntry::from_settings_with_name(&name, &settings, reveal_passwords)
                })
                .collect();
            Ok(entries)
        }
        Err(e) => Err(CommandRunError::FailureDuringExecution {
            message: format!("Failed to read configuration file: {}", e),
        }),
    }
}

fn extract_node_settings_from_args(command_args: &ArgMatches) -> (String, SharedSettings, bool) {
    let base_uri = command_args.get_one::<String>("base_uri").cloned();
    let hostname = command_args.get_one::<String>("host").cloned();
    let port = command_args.get_one::<u16>("port").copied();
    let scheme = command_args.get_one::<String>("scheme").cloned();
    let username = command_args.get_one::<String>("username").cloned();
    let password = command_args.get_one::<String>("password").cloned();
    let vhost = command_args.get_one::<String>("vhost").cloned();
    let path_prefix = command_args.get_one::<String>("path_prefix").cloned();
    let tls = command_args.get_flag("tls");
    let ca_certificate_bundle_path = command_args.get_one::<PathBuf>("tls_ca_cert_file").cloned();
    let client_certificate_file_path = command_args.get_one::<PathBuf>("tls_cert_file").cloned();
    let client_private_key_file_path = command_args.get_one::<PathBuf>("tls_key_file").cloned();

    let node_name = command_args
        .get_one::<String>("node")
        .cloned()
        .or_else(|| hostname.clone())
        .unwrap_or_else(|| DEFAULT_HOST.to_string());

    let settings = SharedSettings {
        base_uri,
        hostname,
        port,
        username,
        password,
        virtual_host: vhost,
        scheme: scheme.map(|s| Scheme::from(s.as_str())).unwrap_or_default(),
        path_prefix: path_prefix.unwrap_or_default(),
        tls,
        ca_certificate_bundle_path,
        client_certificate_file_path,
        client_private_key_file_path,
        ..Default::default()
    };

    let create_file_if_missing = command_args.get_flag("create_file_if_missing");

    (node_name, settings, create_file_if_missing)
}

pub fn config_file_add_node(
    config_path: &Path,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let (node_name, settings, create_file_if_missing) =
        extract_node_settings_from_args(command_args);

    add_node_to_config_file(config_path, &node_name, &settings, create_file_if_missing).map_err(
        |e| CommandRunError::FailureDuringExecution {
            message: format!("Failed to add node to configuration file: {}", e),
        },
    )
}

pub fn config_file_update_node(
    config_path: &Path,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let (node_name, settings, create_file_if_missing) =
        extract_node_settings_from_args(command_args);

    update_node_in_config_file(config_path, &node_name, &settings, create_file_if_missing).map_err(
        |e| CommandRunError::FailureDuringExecution {
            message: format!("Failed to update node in configuration file: {}", e),
        },
    )
}

pub fn config_file_delete_node(
    config_path: &Path,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let node_name = command_args.str_arg("node");
    let create_file_if_missing = command_args.get_flag("create_file_if_missing");

    delete_node_from_config_file(config_path, node_name, create_file_if_missing).map_err(|e| {
        CommandRunError::FailureDuringExecution {
            message: format!("Failed to delete node from configuration file: {}", e),
        }
    })
}
