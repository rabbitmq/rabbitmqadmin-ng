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

use crate::APIClient;
use clap::ArgMatches;

use rabbitmq_http_client::blocking_api::Result as ClientResult;
use rabbitmq_http_client::responses::{SchemaDefinitionSyncStatus, WarmStandbyReplicationStatus};

pub fn sds_status_on_node(
    client: APIClient,
    command_args: &ArgMatches,
) -> ClientResult<SchemaDefinitionSyncStatus> {
    let node = command_args.get_one::<String>("node");
    client.schema_definition_sync_status(node.map(|s| s.as_str()))
}

pub fn sds_enable_cluster_wide(client: APIClient) -> ClientResult<()> {
    client.enable_schema_definition_sync_on_node(None)
}

pub fn sds_disable_cluster_wide(client: APIClient) -> ClientResult<()> {
    client.disable_schema_definition_sync_on_node(None)
}

pub fn sds_enable_on_node(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    let node = command_args.get_one::<String>("node").unwrap();
    client.enable_schema_definition_sync_on_node(Some(node))
}

pub fn sds_disable_on_node(client: APIClient, command_args: &ArgMatches) -> ClientResult<()> {
    let node = command_args.get_one::<String>("node").unwrap();
    client.disable_schema_definition_sync_on_node(Some(node))
}

pub fn wsr_status(client: APIClient) -> ClientResult<WarmStandbyReplicationStatus> {
    client.warm_standby_replication_status()
}
