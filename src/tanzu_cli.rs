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
use clap::{Arg, Command};

pub fn tanzu_subcommands() -> [Command; 2] {
    let sds_cmd = sds_group();
    let wsr_cmd = wsr_group();

    [sds_cmd, wsr_cmd]
}

fn wsr_group() -> Command {
    Command::new("wsr")
        .long_about("Warm Standby Replication (WSR) operations")
        .subcommands(wsr_subcommands())
}

fn sds_group() -> Command {
    Command::new("sds")
        .long_about("Schema Definition Sync (SDS) operations")
        .subcommands(sds_subcommands())
}

fn sds_subcommands() -> [Command; 5] {
    let status_cmd = Command::new("status_on_node")
        .long_about("Reports Schema Definition Sync (SDS) status on the given node")
        .arg(Arg::new("node").short('n').long("node").required(false));

    let disable_on_node_cmd = Command::new("disable_on_node")
        .long_about("Stops Schema Definition Sync (SDS) on the given node")
        .arg(Arg::new("node").short('n').long("node").required(true));

    let enable_on_node_cmd = Command::new("enable_on_node")
        .long_about("Resumes Schema Definition Sync (SDS) on the given node")
        .arg(Arg::new("node").short('n').long("node").required(true));

    let disable_cmd = Command::new("disable_cluster_wide")
        .long_about("Stops Schema Definition Sync (SDS) on all cluster nodes");

    let enable_cmd = Command::new("enable_cluster_wide")
        .long_about("Resumes Schema Definition Sync (SDS) on all cluster nodes");

    [
        status_cmd,
        disable_on_node_cmd,
        disable_cmd,
        enable_on_node_cmd,
        enable_cmd,
    ]
}

fn wsr_subcommands() -> [Command; 1] {
    let status_cmd = Command::new("status")
        .long_about("Reports Warm Standby Replication (WSR) status on the target node");

    [status_cmd]
}
