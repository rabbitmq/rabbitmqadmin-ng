// Copyright (C) 2023-2026 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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

use crate::skip_if_rabbitmq_version_below;
use crate::test_helpers::*;
use predicates::prelude::*;
use std::error::Error;

#[test]
fn test_list_nodes() -> Result<(), Box<dyn Error>> {
    run_succeeds(["list", "nodes"]).stdout(output_includes("rabbit@"));

    run_succeeds(["nodes", "list"]).stdout(output_includes("rabbit@"));

    Ok(())
}

#[test]
fn test_nodes_memory_breakdown_in_bytes_succeeds() -> Result<(), Box<dyn Error>> {
    skip_if_rabbitmq_version_below!(4, 0, 0);

    let rc = api_client();
    let nodes = rc.list_nodes()?;
    let first = nodes.first().unwrap();

    run_succeeds([
        "nodes",
        "memory_breakdown_in_bytes",
        "--node",
        first.name.as_str(),
    ])
    .stdout(
        output_includes("Allocated but unused")
            .and(output_includes("Quorum queue ETS tables"))
            .and(output_includes("Client connections"))
            .and(output_includes("Metadata store")),
    );

    Ok(())
}

#[test]
fn test_nodes_memory_breakdown_in_percent_succeeds() -> Result<(), Box<dyn Error>> {
    skip_if_rabbitmq_version_below!(4, 0, 0);

    let rc = api_client();
    let nodes = rc.list_nodes()?;
    let first = nodes.first().unwrap();

    run_succeeds([
        "nodes",
        "memory_breakdown_in_percent",
        "--node",
        first.name.as_str(),
    ])
    .stdout(
        output_includes("Allocated but unused")
            .and(output_includes("Quorum queue ETS tables"))
            .and(output_includes("Client connections"))
            .and(output_includes("Metadata store")),
    );

    Ok(())
}
