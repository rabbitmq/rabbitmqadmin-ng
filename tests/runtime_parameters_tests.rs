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

use predicates::prelude::*;

mod test_helpers;
use crate::test_helpers::*;

#[test]
fn test_runtime_parameters_across_groups() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_runtime_parameters_across_groups";
    delete_vhost(vh).expect("failed to delete a virtual host");

    run_succeeds(["declare", "vhost", "--name", vh]);
    run_succeeds([
        "-V",
        vh,
        "declare",
        "parameter",
        "--component",
        "federation-upstream",
        "--name",
        "my-upstream",
        "--value",
        "{\"uri\":\"amqp://target.hostname\"}",
    ]);
    await_metric_emission(200);

    run_succeeds([
        "-V",
        vh,
        "list",
        "parameters",
        "--component",
        "federation-upstream",
    ])
    .stdout(predicate::str::contains("my-upstream"));

    run_succeeds([
        "-V",
        vh,
        "delete",
        "parameter",
        "--component",
        "federation-upstream",
        "--name",
        "my-upstream",
    ]);

    run_succeeds([
        "-V",
        vh,
        "list",
        "parameters",
        "--component",
        "federation-upstream",
    ])
    .stdout(predicate::str::contains("my-upstream").not());

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_runtime_parameters_cmd_group() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_runtime_parameters_cmd_group";
    delete_vhost(vh).expect("failed to delete a virtual host");

    run_succeeds(["vhosts", "declare", "--name", vh]);
    run_succeeds([
        "-V",
        vh,
        "parameters",
        "set",
        "--component",
        "federation-upstream",
        "--name",
        "my-upstream",
        "--value",
        "{\"uri\":\"amqp://target.hostname\",\"ack-mode\":\"on-confirm\"}",
    ]);
    await_metric_emission(200);

    run_succeeds(["parameters", "list_all"]).stdout(predicate::str::contains("my-upstream"));

    run_succeeds([
        "-V",
        vh,
        "parameters",
        "list",
        "--component",
        "federation-upstream",
    ])
    .stdout(predicate::str::contains("my-upstream"));

    run_succeeds([
        "-V",
        vh,
        "parameters",
        "list_in",
        "--component",
        "federation-upstream",
    ])
    .stdout(predicate::str::contains("my-upstream"));

    run_succeeds([
        "-V",
        vh,
        "parameters",
        "delete",
        "--component",
        "federation-upstream",
        "--name",
        "my-upstream",
    ]);

    run_succeeds([
        "-V",
        vh,
        "parameters",
        "list",
        "--component",
        "federation-upstream",
    ])
    .stdout(predicate::str::contains("my-upstream").not());

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_global_runtime_parameters_cmd_group() -> Result<(), Box<dyn std::error::Error>> {
    run_succeeds([
        "global_parameters",
        "set",
        "--name",
        "cluster_tags",
        "--value",
        "{\"region\": \"ca-central-1\"}",
    ]);

    run_succeeds(["global_parameters", "list"])
        .stdout(predicate::str::contains("region").and(predicate::str::contains("ca-central-1")));

    run_succeeds(["global_parameters", "delete", "--name", "cluster_tags"]);

    run_succeeds(["global_parameters", "list"])
        .stdout(predicate::str::contains("cluster_tags").not());

    Ok(())
}
