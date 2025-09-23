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

use crate::constants::DEFAULT_BLANKET_POLICY_PRIORITY;
use crate::errors::CommandRunError;
use clap::ArgMatches;
use rabbitmq_http_client::blocking_api::Client;
use rabbitmq_http_client::blocking_api::Result as ClientResult;
use rabbitmq_http_client::commons;
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
    DEFAULT_FEDERATION_PREFETCH, EnforcedLimitParams, ExchangeFederationParams,
    FEDERATION_UPSTREAM_COMPONENT, FederationResourceCleanupMode, FederationUpstreamParams,
    PolicyParams, QueueFederationParams, RuntimeParameterDefinition,
};

use rabbitmq_http_client::transformers::{TransformationChain, VirtualHostTransformationChain};
use rabbitmq_http_client::{password_hashing, requests, responses};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fs;
use std::io;
use std::process;

type APIClient = Client<String, String, String>;

pub fn show_overview(client: APIClient) -> ClientResult<responses::Overview> {
    client.overview()
}

pub fn show_memory_breakdown(
    client: APIClient,
    command_args: &ArgMatches,
) -> ClientResult<responses::NodeMemoryBreakdown> {
    let node = command_args.get_one::<String>("node").unwrap();
    client
        .get_node_memory_footprint(node)
        .map(|footprint| footprint.breakdown)
}

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
    match command_args.get_one::<String>("user") {
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

pub fn list_user_connections(
    client: APIClient,
    command_args: &ArgMatches,
) -> ClientResult<Vec<responses::UserConnection>> {
    let username = command_args.get_one::<String>("username").cloned().unwrap();
    client.list_user_connections(&username)
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

pub fn list_policies_in(client: APIClient, vhost: &str) -> ClientResult<Vec<responses::Policy>> {
    client.list_policies_in(vhost)
}

pub fn list_policies_in_and_applying_to(
    client: APIClient,
    vhost: &str,
    apply_to: PolicyTarget,
) -> ClientResult<Vec<responses::Policy>> {
    let policies = client.list_policies_in(vhost)?;
    Ok(policies
        .into_iter()
        .filter(|pol| apply_to.does_apply_to(pol.apply_to.clone()))
        .collect())
}

pub fn list_matching_policies_in(
    client: APIClient,
    vhost: &str,
    name: &str,
    typ: PolicyTarget,
) -> ClientResult<Vec<responses::Policy>> {
    let candidates = list_policies_in_and_applying_to(client, vhost, typ.clone())?;
    Ok(candidates
        .into_iter()
        .filter(|pol| pol.does_match_name(vhost, name, typ.clone()))
        .collect())
}

pub fn list_operator_policies(client: APIClient) -> ClientResult<Vec<responses::Policy>> {
    client.list_operator_policies()
}

pub fn list_operator_policies_in(
    client: APIClient,
    vhost: &str,
) -> ClientResult<Vec<responses::Policy>> {
    client.list_operator_policies_in(vhost)
}

pub fn list_operator_policies_in_and_applying_to(
    client: APIClient,
    vhost: &str,
    apply_to: PolicyTarget,
) -> ClientResult<Vec<responses::Policy>> {
    let policies = client.list_operator_policies_in(vhost)?;
    Ok(policies
        .into_iter()
        .filter(|pol| apply_to.does_apply_to(pol.apply_to.clone()))
        .collect())
}

pub fn list_matching_operator_policies_in(
    client: APIClient,
    vhost: &str,
    name: &str,
    typ: PolicyTarget,
) -> ClientResult<Vec<responses::Policy>> {
    let candidates = list_operator_policies_in_and_applying_to(client, vhost, typ.clone())?;
    Ok(candidates
        .into_iter()
        .filter(|pol| pol.does_match_name(vhost, name, typ.clone()))
        .collect())
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

pub fn list_all_parameters(client: APIClient) -> ClientResult<Vec<responses::RuntimeParameter>> {
    client.list_runtime_parameters()
}

pub fn list_parameters(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<Vec<responses::RuntimeParameter>> {
    match command_args.get_one::<String>("component") {
        None => {
            let mut r = client.list_runtime_parameters()?;
            r.retain(|p| p.vhost == vhost);
            Ok(r)
        }
        Some(c) => client.list_runtime_parameters_of_component_in(c, vhost),
    }
}

pub fn list_parameters_of_component_in(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<Vec<responses::RuntimeParameter>> {
    let component = command_args.get_one::<String>("component").unwrap();
    client.list_runtime_parameters_of_component_in(component, vhost)
}

pub fn list_global_parameters(
    client: APIClient,
) -> ClientResult<Vec<responses::GlobalRuntimeParameter>> {
    client.list_global_runtime_parameters()
}

pub fn list_feature_flags(client: APIClient) -> ClientResult<responses::FeatureFlagList> {
    client.list_feature_flags()
}

pub fn list_shovels(client: APIClient) -> ClientResult<Vec<responses::Shovel>> {
    client.list_shovels()
}

pub fn list_shovels_in(client: APIClient, vhost: &str) -> ClientResult<Vec<responses::Shovel>> {
    client.list_shovels_in(vhost)
}

pub fn declare_amqp10_shovel(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let name = command_args.get_one::<String>("name").cloned().unwrap();
    let source_uri = command_args
        .get_one::<String>("source_uri")
        .cloned()
        .unwrap();
    let destination_uri = command_args
        .get_one::<String>("destination_uri")
        .cloned()
        .unwrap();

    let source_address = command_args
        .get_one::<String>("source_address")
        .cloned()
        .unwrap();
    let destination_address = command_args
        .get_one::<String>("destination_address")
        .cloned()
        .unwrap();

    let ack_mode = command_args
        .get_one::<MessageTransferAcknowledgementMode>("ack_mode")
        .cloned()
        .unwrap();
    let reconnect_delay = command_args
        .get_one::<u32>("reconnect_delay")
        .cloned()
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

    client.declare_amqp10_shovel(params)
}

pub fn declare_amqp091_shovel(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let name = command_args.get_one::<String>("name").cloned().unwrap();
    let source_uri = command_args
        .get_one::<String>("source_uri")
        .cloned()
        .unwrap();
    let destination_uri = command_args
        .get_one::<String>("destination_uri")
        .cloned()
        .unwrap();

    let ack_mode = command_args
        .get_one::<MessageTransferAcknowledgementMode>("ack_mode")
        .cloned()
        .unwrap();
    let reconnect_delay = command_args
        .get_one::<u32>("reconnect_delay")
        .cloned()
        .or(Some(5));

    let predeclared_source = command_args
        .get_one::<bool>("predeclared_source")
        .cloned()
        .unwrap_or(false);
    let source_queue_opt = command_args.get_one::<String>("source_queue").cloned();
    let source_exchange_opt = command_args.get_one::<String>("source_exchange").cloned();
    let source_exchange_routing_key_opt = command_args
        .get_one::<String>("source_exchange_key")
        .map(|s| s.as_str());

    let predeclared_destination = command_args
        .get_one::<bool>("predeclared_destination")
        .cloned()
        .unwrap_or(false);
    let destination_queue_opt = command_args.get_one::<String>("destination_queue").cloned();
    let destination_exchange_opt = command_args
        .get_one::<String>("destination_exchange")
        .cloned();
    let destination_exchange_routing_key_opt = command_args
        .get_one::<String>("destination_exchange_key")
        .map(|s| s.as_str());

    let source_queue: String;
    let source_exchange: String;
    let source_params = if source_queue_opt.is_some() {
        source_queue = source_queue_opt.unwrap();
        if predeclared_source {
            Amqp091ShovelSourceParams::predeclared_queue_source(&source_uri, &source_queue)
        } else {
            Amqp091ShovelSourceParams::queue_source(&source_uri, &source_queue)
        }
    } else {
        source_exchange = source_exchange_opt.unwrap();
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

    let destination_queue: String;
    let destination_exchange: String;
    let destination_params = if destination_queue_opt.is_some() {
        destination_queue = destination_queue_opt.unwrap();
        if predeclared_destination {
            Amqp091ShovelDestinationParams::predeclared_queue_destination(
                &destination_uri,
                &destination_queue,
            )
        } else {
            Amqp091ShovelDestinationParams::queue_destination(&destination_uri, &destination_queue)
        }
    } else {
        destination_exchange = destination_exchange_opt.unwrap();
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
    client.declare_amqp091_shovel(params)
}

pub fn delete_shovel(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let name = command_args.get_one::<String>("name").cloned().unwrap();

    client.delete_shovel(vhost, &name, true)
}

//
// Federation
//

pub fn list_federation_upstreams(
    client: APIClient,
) -> ClientResult<Vec<responses::FederationUpstream>> {
    client.list_federation_upstreams()
}

pub fn list_federation_links(client: APIClient) -> ClientResult<Vec<responses::FederationLink>> {
    client.list_federation_links()
}

pub fn declare_federation_upstream(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    // common settings
    let name = command_args.get_one::<String>("name").cloned().unwrap();
    let uri = command_args.get_one::<String>("uri").cloned().unwrap();
    let reconnect_delay = command_args
        .get_one::<u32>("reconnect_delay")
        .cloned()
        .unwrap();
    let trust_user_id = command_args
        .get_one::<bool>("trust_user_id")
        .cloned()
        .unwrap();
    let prefetch_count = command_args
        .get_one::<u32>("prefetch_count")
        .cloned()
        .unwrap();
    let ack_mode = command_args
        .get_one::<MessageTransferAcknowledgementMode>("ack_mode")
        .cloned()
        .unwrap();

    // optional queue federation settings
    let queue_name = command_args.get_one::<String>("queue_name").cloned();
    let consumer_tag = command_args.get_one::<String>("consumer_tag").cloned();
    let qn: String;
    let ct: String;
    let qfp = match (queue_name, consumer_tag) {
        (Some(queue_name), Some(consumer_tag)) => {
            qn = queue_name.clone();
            ct = consumer_tag.clone();
            let qfp = QueueFederationParams::new_with_consumer_tag(&qn, &ct);
            Some(qfp)
        }
        (Some(queue_name), None) => {
            qn = queue_name.clone();
            let qfp = QueueFederationParams::new(&qn);
            Some(qfp)
        }
        (None, Some(_)) => None,
        (None, None) => None,
    };

    // optional exchange federation settings
    let exchange_name = command_args
        .get_one::<String>("exchange_name")
        .map(|s| s.as_str());
    let queue_type = command_args
        .get_one::<String>("queue_type")
        .map(|s| Into::<QueueType>::into(s.as_str()))
        .unwrap_or_default();
    let max_hops = command_args.get_one::<u8>("max_hops").copied();
    let resource_cleanup_mode = command_args
        .get_one::<FederationResourceCleanupMode>("resource_cleanup_mode")
        .cloned()
        .unwrap_or_default();
    let bind_using_nowait = command_args
        .get_one::<bool>("bind_nowait")
        .cloned()
        .unwrap_or_default();
    let channel_use_mode = command_args
        .get_one::<ChannelUseMode>("channel_use_mode")
        .cloned()
        .unwrap_or_default();
    let ttl = command_args.get_one::<u32>("ttl").cloned();
    let message_ttl = command_args.get_one::<u32>("message_ttl").cloned();
    let efp = Some(ExchangeFederationParams {
        exchange: exchange_name,
        max_hops,
        queue_type,
        ttl,
        message_ttl,
        resource_cleanup_mode,
    });

    // putting it all together
    let upstream = FederationUpstreamParams {
        name: &name,
        vhost,
        uri: &uri,
        reconnect_delay,
        trust_user_id,
        prefetch_count,
        ack_mode,
        bind_using_nowait,
        channel_use_mode,
        queue_federation: qfp,
        exchange_federation: efp,
    };
    let param = RuntimeParameterDefinition::from(upstream);
    client.upsert_runtime_parameter(&param)
}

pub fn declare_federation_upstream_for_exchange_federation(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let name = command_args.get_one::<String>("name").cloned().unwrap();
    let uri = command_args.get_one::<String>("uri").cloned().unwrap();
    let reconnect_delay = command_args
        .get_one::<u32>("reconnect_delay")
        .cloned()
        .unwrap();
    let trust_user_id = command_args
        .get_one::<bool>("trust_user_id")
        .cloned()
        .unwrap();
    let prefetch_count = command_args
        .get_one::<u32>("prefetch_count")
        .cloned()
        .unwrap();
    let ack_mode = command_args
        .get_one::<MessageTransferAcknowledgementMode>("ack_mode")
        .cloned()
        .unwrap();

    let exchange_name = command_args
        .get_one::<String>("exchange_name")
        .map(|s| s.as_str());
    let queue_type = command_args
        .get_one::<String>("queue_type")
        .map(|s| Into::<QueueType>::into(s.as_str()))
        .unwrap_or_default();
    let max_hops = command_args.get_one::<u8>("max_hops").copied();
    let resource_cleanup_mode = command_args
        .get_one::<FederationResourceCleanupMode>("resource_cleanup_mode")
        .cloned()
        .unwrap_or_default();
    let bind_using_nowait = command_args
        .get_one::<bool>("bind_nowait")
        .cloned()
        .unwrap_or_default();
    let channel_use_mode = command_args
        .get_one::<ChannelUseMode>("channel_use_mode")
        .cloned()
        .unwrap_or_default();
    let ttl = command_args.get_one::<u32>("ttl").cloned();
    let message_ttl = command_args.get_one::<u32>("message_ttl").cloned();
    let efp = Some(ExchangeFederationParams {
        exchange: exchange_name,
        max_hops,
        queue_type,
        ttl,
        message_ttl,
        resource_cleanup_mode,
    });

    // putting it all together
    let upstream = FederationUpstreamParams {
        name: &name,
        vhost,
        uri: &uri,
        reconnect_delay,
        trust_user_id,
        prefetch_count,
        ack_mode,
        bind_using_nowait,
        channel_use_mode,
        queue_federation: None,
        exchange_federation: efp,
    };
    let param = RuntimeParameterDefinition::from(upstream);
    client.upsert_runtime_parameter(&param)
}

pub fn declare_federation_upstream_for_queue_federation(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let name = command_args.get_one::<String>("name").cloned().unwrap();
    let uri = command_args.get_one::<String>("uri").cloned().unwrap();
    let reconnect_delay = command_args
        .get_one::<u32>("reconnect_delay")
        .cloned()
        .unwrap();
    let trust_user_id = command_args
        .get_one::<bool>("trust_user_id")
        .cloned()
        .unwrap();
    let prefetch_count = command_args
        .get_one::<u32>("prefetch_count")
        .cloned()
        .unwrap();
    let ack_mode = command_args
        .get_one::<MessageTransferAcknowledgementMode>("ack_mode")
        .cloned()
        .unwrap();
    let bind_using_nowait = command_args
        .get_one::<bool>("bind_nowait")
        .cloned()
        .unwrap_or_default();
    let channel_use_mode = command_args
        .get_one::<ChannelUseMode>("channel_use_mode")
        .cloned()
        .unwrap_or_default();

    let queue_name = command_args.get_one::<String>("queue_name").cloned();
    let consumer_tag = command_args.get_one::<String>("consumer_tag").cloned();
    let qn: String;
    let ct: String;
    let qfp = match (queue_name, consumer_tag) {
        (Some(queue_name), Some(consumer_tag)) => {
            qn = queue_name.clone();
            ct = consumer_tag.clone();
            let qfp = QueueFederationParams::new_with_consumer_tag(&qn, &ct);
            Some(qfp)
        }
        (Some(queue_name), None) => {
            qn = queue_name.clone();
            let qfp = QueueFederationParams::new(&qn);
            Some(qfp)
        }
        (None, Some(_)) => None,
        (None, None) => None,
    };

    let upstream = FederationUpstreamParams {
        name: &name,
        vhost,
        uri: &uri,
        reconnect_delay,
        trust_user_id,
        prefetch_count,
        ack_mode,
        bind_using_nowait,
        channel_use_mode,
        queue_federation: qfp,
        exchange_federation: None,
    };
    let param = RuntimeParameterDefinition::from(upstream);
    client.upsert_runtime_parameter(&param)
}

pub fn delete_federation_upstream(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let name = command_args.get_one::<String>("name").cloned().unwrap();
    client.clear_runtime_parameter(FEDERATION_UPSTREAM_COMPONENT, vhost, &name)
}

pub fn disable_tls_peer_verification_for_all_federation_upstreams(
    client: APIClient,
) -> Result<(), CommandRunError> {
    let upstreams = client.list_federation_upstreams()?;

    for upstream in upstreams {
        let original_uri = &upstream.uri;
        let updated_uri = disable_tls_peer_verification(original_uri)?;

        if original_uri != &updated_uri {
            let upstream_params = FederationUpstreamParams {
                name: &upstream.name,
                vhost: &upstream.vhost,
                uri: &updated_uri,
                prefetch_count: upstream
                    .prefetch_count
                    .unwrap_or(DEFAULT_FEDERATION_PREFETCH),
                reconnect_delay: upstream.reconnect_delay.unwrap_or(5),
                ack_mode: upstream.ack_mode,
                trust_user_id: upstream.trust_user_id.unwrap_or_default(),
                bind_using_nowait: upstream.bind_using_nowait,
                channel_use_mode: upstream.channel_use_mode,
                queue_federation: if upstream.queue.is_some() {
                    Some(QueueFederationParams {
                        queue: upstream.queue.as_deref(),
                        consumer_tag: upstream.consumer_tag.as_deref(),
                    })
                } else {
                    None
                },
                exchange_federation: if upstream.exchange.is_some() {
                    Some(ExchangeFederationParams {
                        exchange: upstream.exchange.as_deref(),
                        max_hops: upstream.max_hops,
                        queue_type: upstream.queue_type.unwrap_or(QueueType::Classic),
                        ttl: upstream.expires,
                        message_ttl: upstream.message_ttl,
                        resource_cleanup_mode: upstream.resource_cleanup_mode,
                    })
                } else {
                    None
                },
            };

            let param = RuntimeParameterDefinition::from(upstream_params);
            client.upsert_runtime_parameter(&param)?;
        }
    }

    Ok(())
}

pub fn enable_tls_peer_verification_for_all_federation_upstreams(
    client: APIClient,
    args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let ca_cert_path = args
        .get_one::<String>("node_local_ca_certificate_bundle_path")
        .ok_or_else(|| CommandRunError::MissingArgumentValue {
            property: "node_local_ca_certificate_bundle_path".to_string(),
        })?;
    let client_cert_path = args
        .get_one::<String>("node_local_client_certificate_file_path")
        .ok_or_else(|| CommandRunError::MissingArgumentValue {
            property: "node_local_client_certificate_file_path".to_string(),
        })?;
    let client_key_path = args
        .get_one::<String>("node_local_client_private_key_file_path")
        .ok_or_else(|| CommandRunError::MissingArgumentValue {
            property: "node_local_client_private_key_file_path".to_string(),
        })?;

    let upstreams = client.list_federation_upstreams()?;

    for upstream in upstreams {
        let original_uri = &upstream.uri;
        let updated_uri = enable_tls_peer_verification(
            original_uri,
            ca_cert_path,
            client_cert_path,
            client_key_path,
        )?;

        if original_uri != &updated_uri {
            let upstream_params = FederationUpstreamParams {
                name: &upstream.name,
                vhost: &upstream.vhost,
                uri: &updated_uri,
                prefetch_count: upstream
                    .prefetch_count
                    .unwrap_or(DEFAULT_FEDERATION_PREFETCH),
                reconnect_delay: upstream.reconnect_delay.unwrap_or(5),
                ack_mode: upstream.ack_mode,
                trust_user_id: upstream.trust_user_id.unwrap_or_default(),
                bind_using_nowait: upstream.bind_using_nowait,
                channel_use_mode: upstream.channel_use_mode,
                queue_federation: if upstream.queue.is_some() {
                    Some(QueueFederationParams {
                        queue: upstream.queue.as_deref(),
                        consumer_tag: upstream.consumer_tag.as_deref(),
                    })
                } else {
                    None
                },
                exchange_federation: if upstream.exchange.is_some() {
                    Some(ExchangeFederationParams {
                        exchange: upstream.exchange.as_deref(),
                        max_hops: upstream.max_hops,
                        queue_type: upstream.queue_type.unwrap_or(QueueType::Classic),
                        ttl: upstream.expires,
                        message_ttl: upstream.message_ttl,
                        resource_cleanup_mode: upstream.resource_cleanup_mode,
                    })
                } else {
                    None
                },
            };

            let param = RuntimeParameterDefinition::from(upstream_params);
            client.upsert_runtime_parameter(&param)?;
        }
    }

    Ok(())
}

pub fn disable_tls_peer_verification_for_all_source_uris(
    client: APIClient,
) -> Result<(), CommandRunError> {
    let all_params = client.list_runtime_parameters()?;
    let shovel_params: Vec<_> = all_params
        .into_iter()
        .filter(|p| p.component == "shovel")
        .collect();

    for param in shovel_params {
        let owned_params = match OwnedShovelParams::try_from(param.clone()) {
            Ok(params) => params,
            Err(_) => continue,
        };

        let original_source_uri = &owned_params.source_uri;

        if original_source_uri.is_empty() {
            continue;
        }

        let updated_source_uri = disable_tls_peer_verification(original_source_uri)?;

        if original_source_uri != &updated_source_uri {
            let mut updated_params = owned_params;
            updated_params.source_uri = updated_source_uri;

            let param = RuntimeParameterDefinition::from(&updated_params);
            client.upsert_runtime_parameter(&param)?;
        }
    }

    Ok(())
}

pub fn disable_tls_peer_verification_for_all_destination_uris(
    client: APIClient,
) -> Result<(), CommandRunError> {
    let all_params = client.list_runtime_parameters()?;
    let shovel_params: Vec<_> = all_params
        .into_iter()
        .filter(|p| p.component == "shovel")
        .collect();

    for param in shovel_params {
        let owned_params = match OwnedShovelParams::try_from(param.clone()) {
            Ok(params) => params,
            Err(_) => continue,
        };

        let original_destination_uri = &owned_params.destination_uri;

        if original_destination_uri.is_empty() {
            continue;
        }

        let updated_destination_uri = disable_tls_peer_verification(original_destination_uri)?;

        if original_destination_uri != &updated_destination_uri {
            let mut updated_params = owned_params;
            updated_params.destination_uri = updated_destination_uri;

            let param = RuntimeParameterDefinition::from(&updated_params);
            client.upsert_runtime_parameter(&param)?;
        }
    }

    Ok(())
}

//
// Feature flags
//

pub fn enable_feature_flag(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    let name = command_args.get_one::<String>("name").cloned().unwrap();
    client.enable_feature_flag(&name)
}

pub fn enable_all_stable_feature_flags(client: APIClient) -> ClientResult<()> {
    client.enable_all_stable_feature_flags()
}

//
// Deprecated features
//

pub fn list_deprecated_features(
    client: APIClient,
) -> ClientResult<responses::DeprecatedFeatureList> {
    client.list_all_deprecated_features()
}

pub fn list_deprecated_features_in_use(
    client: APIClient,
) -> ClientResult<responses::DeprecatedFeatureList> {
    client.list_deprecated_features_in_use()
}

//
// Declaration of core resources
//

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
    let tracing = command_args
        .get_one::<bool>("tracing")
        .cloned()
        .unwrap_or(false);

    let params = requests::VirtualHostParams {
        name,
        description,
        default_queue_type: dqt,
        tags: None,
        tracing,
    };

    client.create_vhost(&params)
}

pub fn declare_exchange(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    // these are optional
    let exchange_type = command_args
        .get_one::<ExchangeType>("type")
        .cloned()
        .unwrap_or(commons::ExchangeType::Direct);
    let durable = command_args
        .get_one::<bool>("durable")
        .cloned()
        .unwrap_or(true);
    let auto_delete = command_args
        .get_one::<bool>("auto_delete")
        .cloned()
        .unwrap_or(false);
    let arguments = command_args.get_one::<String>("arguments").unwrap();

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
    let source = command_args.get_one::<String>("source").unwrap();
    let destination_type = command_args
        .get_one::<BindingDestinationType>("destination_type")
        .unwrap();
    let destination = command_args.get_one::<String>("destination").unwrap();
    let routing_key = command_args.get_one::<String>("routing_key").unwrap();
    let arguments = command_args.get_one::<String>("arguments").unwrap();
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

pub fn delete_global_parameter(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    let name = command_args.get_one::<String>("name").unwrap();

    client.clear_global_runtime_parameter(name)
}

pub fn delete_vhost(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    let idempotently = command_args
        .get_one::<bool>("idempotently")
        .cloned()
        .unwrap_or(false);
    client.delete_vhost(name, idempotently)
}

pub fn delete_user(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    let idempotently = command_args
        .get_one::<bool>("idempotently")
        .cloned()
        .unwrap_or(false);
    client.delete_user(name, idempotently)
}

pub fn delete_permissions(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    // the flag is required
    let user = command_args.get_one::<String>("user").unwrap();
    let idempotently = command_args
        .get_one::<bool>("idempotently")
        .cloned()
        .unwrap_or(false);
    client.clear_permissions(vhost, user, idempotently)
}

pub fn declare_user(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    let name = command_args.get_one::<String>("name").unwrap();
    let password = command_args.get_one::<String>("password").unwrap();
    let provided_hash = command_args.get_one::<String>("password_hash").unwrap();
    let tags = command_args.get_one::<String>("tags").unwrap();

    let has_password = !password.is_empty();
    let has_hash = !provided_hash.is_empty();

    if !has_password && !has_hash {
        eprintln!("Please provide either --password or --password-hash");
        process::exit(1);
    }

    if has_password && has_hash {
        eprintln!("Please provide either --password or --password-hash");
        process::exit(1);
    }

    let password_hash = if provided_hash.is_empty() {
        let hashing_algo = command_args
            .get_one::<HashingAlgorithm>("hashing_algorithm")
            .unwrap();
        let salt = password_hashing::salt();
        let hash = hashing_algo.salt_and_hash(&salt, password).unwrap();
        String::from_utf8(hash.into()).unwrap()
    } else {
        provided_hash.to_owned()
    };

    let params = requests::UserParams {
        name,
        password_hash: password_hash.as_str(),
        tags,
    };
    client.create_user(&params)
}

pub fn salt_and_hash_password(command_args: &ArgMatches) -> Result<String, HashingError> {
    let password = command_args.get_one::<String>("password").cloned().unwrap();
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
) -> Result<(), CommandRunError> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    let queue_type = command_args.get_one::<QueueType>("type").cloned().unwrap();

    // these are optional
    let durable = command_args
        .get_one::<bool>("durable")
        .cloned()
        .unwrap_or(true);
    let auto_delete = command_args
        .get_one::<bool>("auto_delete")
        .cloned()
        .unwrap_or(false);
    let arguments = command_args.get_one::<String>("arguments").unwrap();
    let parsed_args = parse_json_from_arg(arguments)?;

    let params = requests::QueueParams::new(name, queue_type, durable, auto_delete, parsed_args);

    client.declare_queue(vhost, &params).map_err(Into::into)
}

pub fn declare_stream(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.get_one::<String>("name").unwrap();
    let expiration = command_args.get_one::<String>("expiration").unwrap();
    let max_length_bytes = command_args.get_one::<u64>("max_length_bytes").cloned();
    let max_segment_length_bytes = command_args
        .get_one::<u64>("max_segment_length_bytes")
        .cloned();
    let arguments = command_args.get_one::<String>("arguments").unwrap();
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
    let name = command_args.get_one::<String>("name").unwrap();
    let pattern = command_args.get_one::<String>("pattern").unwrap();
    let apply_to = command_args
        .get_one::<PolicyTarget>("apply_to")
        .cloned()
        .unwrap();
    let priority = command_args.get_one::<String>("priority").unwrap();
    let definition = command_args.get_one::<String>("definition").unwrap();

    let parsed_definition = parse_json_from_arg(definition)?;

    let params = PolicyParams {
        vhost,
        name,
        pattern,
        apply_to: apply_to.clone(),
        priority: priority.parse::<i32>().unwrap(),
        definition: parsed_definition,
    };

    client.declare_policy(&params).map_err(Into::into)
}

pub fn declare_operator_policy(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.get_one::<String>("name").cloned().unwrap();
    let pattern = command_args.get_one::<String>("pattern").cloned().unwrap();
    let apply_to = command_args
        .get_one::<PolicyTarget>("apply_to")
        .cloned()
        .unwrap();
    let priority = command_args.get_one::<String>("priority").unwrap();
    let definition = command_args.get_one::<String>("definition").unwrap();

    let parsed_definition = parse_json_from_arg(definition)?;

    let params = PolicyParams {
        vhost,
        name: &name,
        pattern: &pattern,
        apply_to: apply_to.clone(),
        priority: priority.parse::<i32>().unwrap(),
        definition: parsed_definition,
    };

    client.declare_operator_policy(&params).map_err(Into::into)
}

pub fn declare_policy_override(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let original_pol_name = command_args.get_one::<String>("name").cloned().unwrap();
    let override_pol_name = command_args
        .get_one::<String>("override_name")
        .cloned()
        .unwrap_or(override_policy_name(&original_pol_name));

    let existing_policy = client
        .get_policy(vhost, &original_pol_name)
        .map_err(CommandRunError::from)?;

    let new_priority = existing_policy.priority + 100;
    let definition = command_args.get_one::<String>("definition").unwrap();

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
    // find the lowest policy priority in the target virtual host
    let existing_policies = client
        .list_policies_in(vhost)
        .map_err(CommandRunError::from)?;
    let min_priority = existing_policies
        .iter()
        .fold(0, |acc, p| if p.priority < acc { p.priority } else { acc });

    // blanket policy priority should be the lowest in the virtual host
    let priority = [min_priority - 1, DEFAULT_BLANKET_POLICY_PRIORITY]
        .iter()
        .min()
        .cloned()
        .unwrap();

    let name = command_args.get_one::<String>("name").cloned().unwrap();
    let apply_to = command_args
        .get_one::<PolicyTarget>("apply_to")
        .cloned()
        .unwrap();
    let definition = command_args.get_one::<String>("definition").unwrap();

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
    let name = command_args.get_one::<String>("name").cloned().unwrap();
    let key = command_args
        .get_one::<String>("definition_key")
        .cloned()
        .unwrap();
    let value = command_args
        .get_one::<String>("definition_value")
        .cloned()
        .unwrap();
    let parsed_value = parse_json_from_arg::<serde_json::Value>(&value)?;

    update_policy_definition_with(&client, vhost, &name, &key, &parsed_value).map_err(Into::into)
}

pub fn update_operator_policy_definition(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.get_one::<String>("name").cloned().unwrap();
    let key = command_args
        .get_one::<String>("definition_key")
        .cloned()
        .unwrap();
    let value = command_args
        .get_one::<String>("definition_value")
        .cloned()
        .unwrap();
    let parsed_value = parse_json_from_arg::<serde_json::Value>(&value)?;

    update_operator_policy_definition_with(&client, vhost, &name, &key, &parsed_value)
        .map_err(Into::into)
}

pub fn patch_policy_definition(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let name = command_args.get_one::<String>("name").cloned().unwrap();
    let value = command_args
        .get_one::<String>("definition")
        .cloned()
        .unwrap();
    let parsed_value = parse_json_from_arg::<serde_json::Value>(&value)?;

    let mut pol = client
        .get_policy(vhost, &name)
        .map_err(CommandRunError::from)?;
    let patch = parsed_value.as_object().unwrap();
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
    let key = command_args
        .get_one::<String>("definition_key")
        .cloned()
        .unwrap();
    let value = command_args
        .get_one::<String>("definition_value")
        .cloned()
        .unwrap();
    let parsed_value = parse_json_from_arg::<serde_json::Value>(&value)?;

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
    let name = command_args.get_one::<String>("name").cloned().unwrap();
    let value = command_args
        .get_one::<String>("definition")
        .cloned()
        .unwrap();
    let parsed_value = parse_json_from_arg::<serde_json::Value>(&value)?;

    let mut pol = client
        .get_operator_policy(vhost, &name)
        .map_err(CommandRunError::from)?;
    let patch = parsed_value.as_object().unwrap();
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
    let key = command_args
        .get_one::<String>("definition_key")
        .cloned()
        .unwrap();
    let value = command_args
        .get_one::<String>("definition_value")
        .cloned()
        .unwrap();
    let parsed_value = parse_json_from_arg::<serde_json::Value>(&value)?;

    for pol in pols {
        update_operator_policy_definition_with(&client, vhost, &pol.name, &key, &parsed_value)?
    }

    Ok(())
}

pub fn delete_policy_definition_keys(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let name = command_args.get_one::<String>("name").cloned().unwrap();
    let keys = command_args
        .get_many::<String>("definition_keys")
        .unwrap()
        .map(String::from)
        .collect::<Vec<_>>();
    let str_keys: Vec<&str> = keys.iter().map(AsRef::as_ref).collect::<Vec<_>>();

    let pol = client.get_policy(vhost, &name)?;
    let updated_pol = pol.without_keys(str_keys);

    let params = PolicyParams::from(&updated_pol);
    client.declare_policy(&params)
}

pub fn delete_policy_definition_keys_in(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let pols = client.list_policies_in(vhost)?;
    let keys = command_args
        .get_many::<String>("definition_keys")
        .unwrap()
        .map(String::from)
        .collect::<Vec<_>>();
    let str_keys: Vec<&str> = keys.iter().map(AsRef::as_ref).collect::<Vec<_>>();

    for pol in pols {
        let updated_pol = pol.without_keys(str_keys.clone());

        let params = PolicyParams::from(&updated_pol);
        client.declare_policy(&params)?
    }

    Ok(())
}

pub fn delete_operator_policy_definition_keys(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let name = command_args.get_one::<String>("name").cloned().unwrap();
    let keys = command_args
        .get_many::<String>("definition_keys")
        .unwrap()
        .map(String::from)
        .collect::<Vec<_>>();
    let str_keys: Vec<&str> = keys.iter().map(AsRef::as_ref).collect::<Vec<_>>();

    let pol = client.get_operator_policy(vhost, &name)?;
    let updated_pol = pol.without_keys(str_keys);

    let params = PolicyParams::from(&updated_pol);
    client.declare_operator_policy(&params)
}

pub fn delete_operator_policy_definition_keys_in(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    let pols = client.list_operator_policies_in(vhost)?;
    let keys = command_args
        .get_many::<String>("definition_keys")
        .unwrap()
        .map(String::from)
        .collect::<Vec<_>>();
    let str_keys: Vec<&str> = keys.iter().map(AsRef::as_ref).collect::<Vec<_>>();

    for pol in pols {
        let updated_pol = pol.without_keys(str_keys.clone());

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
) -> ClientResult<()> {
    let mut policy = client.get_policy(vhost, name)?;
    policy.insert_definition_key(key.to_owned(), parsed_value.clone());

    let params = PolicyParams::from(&policy);
    client.declare_policy(&params)
}

fn update_operator_policy_definition_with(
    client: &APIClient,
    vhost: &str,
    name: &str,
    key: &str,
    parsed_value: &Value,
) -> ClientResult<()> {
    let mut policy = client.get_operator_policy(vhost, name)?;
    policy.insert_definition_key(key.to_owned(), parsed_value.clone());

    let params = PolicyParams::from(&policy);
    client.declare_operator_policy(&params)
}

pub fn declare_parameter(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let component = command_args.get_one::<String>("component").unwrap();
    let name = command_args.get_one::<String>("name").unwrap();
    let value = command_args.get_one::<String>("value").unwrap();
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
    let name = command_args.get_one::<String>("name").unwrap();
    let value = command_args.get_one::<String>("value").unwrap();
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

pub fn delete_queue(client: APIClient, vhost: &str, command_args: &ArgMatches) -> ClientResult<()> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    let idempotently = command_args
        .get_one::<bool>("idempotently")
        .cloned()
        .unwrap_or(false);
    client.delete_queue(vhost, name, idempotently)
}

pub fn delete_stream(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    delete_queue(client, vhost, command_args)
}

pub fn delete_binding(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<(), CommandRunError> {
    let source = command_args.get_one::<String>("source").unwrap();
    let destination_type = command_args.get_one::<String>("destination_type").unwrap();
    let destination = command_args.get_one::<String>("destination").unwrap();
    let routing_key = command_args.get_one::<String>("routing_key").unwrap();
    let arguments = command_args.get_one::<String>("arguments").unwrap();
    let parsed_arguments = parse_json_from_arg(arguments)?;

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
        .map_err(Into::into)
}

pub fn delete_exchange(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    let idempotent = command_args
        .get_one::<bool>("idempotently")
        .cloned()
        .unwrap_or(false);
    client.delete_exchange(vhost, name, idempotent)
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

pub fn health_check_local_alarms(client: APIClient) -> ClientResult<()> {
    client.health_check_local_alarms()
}

pub fn health_check_cluster_wide_alarms(client: APIClient) -> ClientResult<()> {
    client.health_check_cluster_wide_alarms()
}

pub fn health_check_node_is_quorum_critical(client: APIClient) -> ClientResult<()> {
    client.health_check_if_node_is_quorum_critical()
}

pub fn health_check_port_listener(
    client: APIClient,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    // the flag is required
    let port = command_args.get_one::<u16>("port").cloned().unwrap();
    client.health_check_port_listener(port)
}

pub fn health_check_protocol_listener(
    client: APIClient,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    // the flag is required
    let proto = command_args
        .get_one::<SupportedProtocol>("protocol")
        .cloned()
        .unwrap();
    client.health_check_protocol_listener(proto)
}

pub fn close_connection(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    // the flag is required
    let name = command_args.get_one::<String>("name").unwrap();
    client.close_connection(name, Some("closed via rabbitmqadmin v2"))
}

pub fn close_user_connections(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    // the flag is required
    let username = command_args.get_one::<String>("username").unwrap();
    client.close_user_connections(username, Some("closed via rabbitmqadmin v2"))
}

pub fn rebalance_queues(client: APIClient) -> ClientResult<()> {
    client.rebalance_queue_leaders()
}

pub fn export_cluster_wide_definitions(
    client: APIClient,
    command_args: &ArgMatches,
) -> ClientResult<()> {
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
) -> ClientResult<()> {
    match client.export_cluster_wide_definitions_as_data() {
        Ok(mut defs0) => {
            let chain = TransformationChain::from(transformations);
            let defs1 = chain.apply(&mut defs0);
            let json = serde_json::to_string_pretty(&defs1).unwrap();

            let path = command_args.get_one::<String>("file").unwrap();
            match path.as_str() {
                "-" => {
                    println!("{}", &json);
                    Ok(())
                }
                file => {
                    _ = fs::write(file, &json);
                    Ok(())
                }
            }
        }
        Err(err) => Err(err),
    }
}

fn export_cluster_wide_definitions_without_transformations(
    client: APIClient,
    command_args: &ArgMatches,
) -> ClientResult<()> {
    match client.export_cluster_wide_definitions() {
        Ok(definitions) => {
            let path = command_args.get_one::<String>("file").cloned();
            let use_stdout = command_args.get_one::<bool>("stdout").copied();
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
                        _ = fs::write(val, &definitions);
                        Ok(())
                    }
                },
                (_, Some(true)) => {
                    println!("{}", &definitions);
                    Ok(())
                }
                _ => {
                    eprintln!("either --file or --stdout must be provided");
                    process::exit(1)
                }
            }
        }
        Err(err) => Err(err),
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

            let path = command_args.get_one::<String>("file").unwrap();
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
            let path = command_args.get_one::<String>("file").unwrap();
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
    let path = command_args
        .get_one::<String>("file")
        .map(|s| s.trim_ascii().trim_matches('\'').trim_matches('"'));
    let use_stdin = command_args.get_one::<bool>("stdin").copied();
    let definitions = read_definitions(path, use_stdin).map_err(|err| {
        let message = match path {
            None => format!("could not read from standard input: {}", err),
            Some(val) => format!("`{}` does not exist or is not readable: {}", val, err),
        };
        CommandRunError::FailureDuringExecution { message }
    })?;

    serde_json::from_str(definitions.as_str()).map_err(|err| {
        let message = match path {
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
            read_stdin_lines(&mut buffer);

            Ok(buffer)
        }
        (Some(val), _) => match val {
            "-" => {
                let mut buffer = String::new();
                read_stdin_lines(&mut buffer);

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

fn read_stdin_lines(buffer: &mut String) {
    let stdin = io::stdin();
    let lines = stdin.lines();
    for ln in lines {
        match ln {
            Ok(line) => buffer.push_str(&line),
            Err(err) => {
                eprintln!("Error reading from standard input: {}", err);
                process::exit(1);
            }
        }
    }
}

pub fn publish_message(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> Result<responses::MessageRouted, CommandRunError> {
    let exchange = command_args.get_one::<String>("exchange").unwrap();
    let routing_key = command_args.get_one::<String>("routing_key").unwrap();
    let payload = command_args.get_one::<String>("payload").unwrap();
    let properties = command_args.get_one::<String>("properties").unwrap();
    let parsed_properties = parse_json_from_arg(properties)?;

    client
        .publish_message(vhost, exchange, routing_key, payload, parsed_properties)
        .map_err(Into::into)
}

pub fn get_messages(
    client: APIClient,
    vhost: &str,
    command_args: &ArgMatches,
) -> ClientResult<Vec<responses::GetMessage>> {
    let queue = command_args.get_one::<String>("queue").unwrap();
    let count = command_args.get_one::<String>("count").unwrap();
    let ack_mode = command_args.get_one::<String>("ack_mode").unwrap();
    client.get_messages(vhost, queue, count.parse::<u32>().unwrap(), ack_mode)
}

fn parse_json_from_arg<T: DeserializeOwned>(input: &str) -> Result<T, CommandRunError> {
    serde_json::from_str(input).map_err(|err| CommandRunError::JsonParseError {
        message: format!("`{}` is not a valid JSON: {}", input, err),
    })
}

fn disable_tls_peer_verification(uri: &str) -> Result<String, CommandRunError> {
    use rabbitmq_http_client::uris::UriBuilder;

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

fn enable_tls_peer_verification(
    uri: &str,
    ca_cert_path: &str,
    client_cert_path: &str,
    client_key_path: &str,
) -> Result<String, CommandRunError> {
    use rabbitmq_http_client::uris::UriBuilder;

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
