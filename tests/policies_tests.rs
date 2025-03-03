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
fn test_list_policies() -> Result<(), Box<dyn std::error::Error>> {
    let policy_name = "test_policy";

    run_succeeds([
        "declare",
        "policy",
        "--name",
        policy_name,
        "--pattern",
        "foo-.*",
        "--apply-to",
        "queues",
        "--priority",
        "123",
        "--definition",
        "{\"max-length\": 12345}",
    ]);

    run_succeeds(["list", "policies"])
        .stdout(predicate::str::contains(policy_name).and(predicate::str::contains("12345")));
    run_succeeds(["delete", "policy", "--name", policy_name]);
    run_succeeds(["list", "policies"]).stdout(predicate::str::contains(policy_name).not());

    Ok(())
}

#[test]
fn test_operator_policies() -> Result<(), Box<dyn std::error::Error>> {
    let operator_policy_name = "test_operator_policy";

    run_succeeds([
        "declare",
        "operator_policy",
        "--name",
        operator_policy_name,
        "--pattern",
        "op-foo.*",
        "--apply-to",
        "queues",
        "--priority",
        "123",
        "--definition",
        "{\"max-length\": 12345}",
    ]);

    run_succeeds(["list", "operator_policies"]).stdout(
        predicate::str::contains(operator_policy_name).and(predicate::str::contains("op-foo")),
    );
    run_succeeds(["delete", "operator_policy", "--name", operator_policy_name]);
    run_succeeds(["list", "operator_policies"])
        .stdout(predicate::str::contains(operator_policy_name).not());

    Ok(())
}

#[test]
fn test_policies_declare_list_and_delete() -> Result<(), Box<dyn std::error::Error>> {
    let policy_name = "test_policies_declare_list_and_delete";

    run_succeeds([
        "policies",
        "declare",
        "--name",
        policy_name,
        "--pattern",
        "foo-.*",
        "--apply-to",
        "queues",
        "--priority",
        "123",
        "--definition",
        "{\"max-length\": 20}",
    ]);

    run_succeeds(["policies", "list"])
        .stdout(predicate::str::contains(policy_name).and(predicate::str::contains("20")));
    run_succeeds(["policies", "delete", "--name", policy_name]);
    run_succeeds(["policies", "list"]).stdout(predicate::str::contains(policy_name).not());

    Ok(())
}

#[test]
fn test_policies_in() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "rabbitmqadmin.vh.policies.1";
    run_succeeds(["delete", "vhost", "--name", vh, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh]);

    let policy_name = "test_policies_in";
    run_succeeds([
        "--vhost",
        vh,
        "policies",
        "declare",
        "--name",
        policy_name,
        "--pattern",
        "foo-.*",
        "--apply-to",
        "queues",
        "--priority",
        "98",
        "--definition",
        "{\"max-length\": 20}",
    ]);

    run_succeeds(["--vhost", vh, "policies", "list_in"])
        .stdout(predicate::str::contains(policy_name).and(predicate::str::contains("98")));
    run_succeeds(["--vhost", "/", "policies", "list_in"])
        .stdout(predicate::str::contains(policy_name).not());
    run_succeeds(["--vhost", vh, "policies", "delete", "--name", policy_name]);
    run_succeeds(["--vhost", vh, "policies", "list_in"])
        .stdout(predicate::str::contains(policy_name).not());

    run_succeeds(["delete", "vhost", "--name", vh]);

    Ok(())
}

#[test]
fn test_policies_in_with_entity_type() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "rabbitmqadmin.vh.policies.2";
    run_succeeds(["delete", "vhost", "--name", vh, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh]);

    let policy_name = "test_policies_in_with_entity_type";
    run_succeeds([
        "--vhost",
        vh,
        "policies",
        "declare",
        "--name",
        policy_name,
        "--pattern",
        "foo-.*",
        "--apply-to",
        "queues",
        "--priority",
        "98",
        "--definition",
        "{\"max-length\": 20}",
    ]);

    run_succeeds(["--vhost", vh, "policies", "list_in", "--apply-to", "queues"])
        .stdout(predicate::str::contains(policy_name).and(predicate::str::contains("98")));
    run_succeeds([
        "--vhost",
        vh,
        "policies",
        "list_in",
        "--apply-to",
        "exchanges",
    ])
    .stdout(predicate::str::contains(policy_name).not());
    run_succeeds([
        "--vhost",
        vh,
        "policies",
        "list_in",
        "--apply-to",
        "streams",
    ])
    .stdout(predicate::str::contains(policy_name).not());
    run_succeeds([
        "--vhost",
        "/",
        "policies",
        "list_in",
        "--apply-to",
        "queues",
    ])
    .stdout(predicate::str::contains(policy_name).not());
    run_succeeds(["--vhost", vh, "policies", "delete", "--name", policy_name]);
    run_succeeds(["--vhost", vh, "policies", "list_in"])
        .stdout(predicate::str::contains(policy_name).not());

    run_succeeds(["delete", "vhost", "--name", vh]);

    Ok(())
}
