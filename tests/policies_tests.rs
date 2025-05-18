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

#[test]
fn test_policies_matching_objects() -> Result<(), Box<dyn std::error::Error>> {
    let vh1 = "rabbitmqadmin.vh.policies.11";
    let vh2 = "rabbitmqadmin.vh.policies.12";
    let vh3 = "rabbitmqadmin.vh.policies.13";

    run_succeeds(["delete", "vhost", "--name", vh1, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh1]);
    run_succeeds(["delete", "vhost", "--name", vh2, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh2]);
    run_succeeds(["delete", "vhost", "--name", vh3, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh3]);

    let policy1 = "rabbitmqadmin.policies.11";
    run_succeeds([
        "--vhost",
        vh1,
        "policies",
        "declare",
        "--name",
        policy1,
        "--pattern",
        "^q-.*",
        "--apply-to",
        "queues",
        "--priority",
        "47",
        "--definition",
        "{\"max-length\": 20}",
    ]);

    let policy2 = "rabbitmqadmin.policies.12";
    run_succeeds([
        "--vhost",
        vh2,
        "policies",
        "declare",
        "--name",
        policy2,
        "--pattern",
        "^x-.*",
        "--apply-to",
        "exchanges",
        "--priority",
        "17",
        "--definition",
        "{\"alternate-exchange\": \"amq.fanout\"}",
    ]);

    let policy3 = "rabbitmqadmin.policies.13";
    run_succeeds([
        "--vhost",
        vh3,
        "policies",
        "declare",
        "--name",
        policy3,
        "--pattern",
        "^s-.*",
        "--apply-to",
        "streams",
        "--priority",
        "15",
        "--definition",
        "{\"max-age\": \"1D\"}",
    ]);

    run_succeeds([
        "--vhost",
        vh1,
        "policies",
        "list_matching_object",
        "--name",
        "q-abc",
        "--type",
        "queues",
    ])
    .stdout(predicate::str::contains(policy1).and(predicate::str::contains("20")));
    run_succeeds([
        "--vhost",
        vh1,
        "policies",
        "list_matching_object",
        "--name",
        "q-abc",
        "--type",
        "exchanges",
    ])
    .stdout(predicate::str::contains(policy1).not());

    run_succeeds([
        "--vhost",
        vh2,
        "policies",
        "list_matching_object",
        "--name",
        "x-abc",
        "--type",
        "exchanges",
    ])
    .stdout(predicate::str::contains(policy2));
    run_succeeds([
        "--vhost",
        vh2,
        "policies",
        "list_matching_object",
        "--name",
        "x-abc",
        "--type",
        "streams",
    ])
    .stdout(predicate::str::contains(policy2).not());

    run_succeeds([
        "--vhost",
        vh3,
        "policies",
        "list_matching_object",
        "--name",
        "s-abc",
        "--type",
        "streams",
    ])
    .stdout(predicate::str::contains(policy3).and(predicate::str::contains("1D")));
    run_succeeds([
        "--vhost",
        vh3,
        "policies",
        "list_matching_object",
        "--name",
        "s-abc",
        "--type",
        "exchanges",
    ])
    .stdout(predicate::str::contains(policy3).not());

    run_succeeds(["delete", "vhost", "--name", vh1, "--idempotently"]);
    run_succeeds(["delete", "vhost", "--name", vh2, "--idempotently"]);
    run_succeeds(["delete", "vhost", "--name", vh3, "--idempotently"]);

    Ok(())
}

#[test]
fn test_policies_declare_list_update_definition_and_delete()
-> Result<(), Box<dyn std::error::Error>> {
    let policy_name = "test_policies_declare_list_update_definition_and_delete";

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

    run_succeeds([
        "policies",
        "update_definition",
        "--name",
        policy_name,
        "--definition-key",
        "max-length",
        "--new-value",
        "131",
    ]);

    run_succeeds(["policies", "list"])
        .stdout(predicate::str::contains(policy_name).and(predicate::str::contains("131")));

    run_succeeds(["policies", "delete", "--name", policy_name]);
    run_succeeds(["policies", "list"]).stdout(predicate::str::contains(policy_name).not());

    Ok(())
}

#[test]
fn test_policies_individual_policy_key_manipulation() -> Result<(), Box<dyn std::error::Error>> {
    let policy_name = "test_policies_individual_policy_key_manipulation";

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
        "{\"max-length\": 20, \"max-length-bytes\": 99999999}",
    ]);
    run_succeeds(["policies", "list"])
        .stdout(predicate::str::contains(policy_name).and(predicate::str::contains("20")));

    run_succeeds([
        "policies",
        "update_definition",
        "--name",
        policy_name,
        "--definition-key",
        "max-length",
        "--new-value",
        "131",
    ]);

    run_succeeds(["policies", "list"])
        .stdout(predicate::str::contains(policy_name).and(predicate::str::contains("131")));

    run_succeeds([
        "policies",
        "delete_definition_key",
        "--name",
        policy_name,
        "--definition-key",
        "max-length",
    ]);

    run_succeeds(["policies", "list"])
        .stdout(predicate::str::contains(policy_name).and(predicate::str::contains("99999999")));

    run_succeeds(["policies", "list"]).stdout(predicate::str::contains("131").not());

    run_succeeds(["policies", "delete", "--name", policy_name]);
    run_succeeds(["policies", "list"]).stdout(predicate::str::contains(policy_name).not());

    Ok(())
}

#[test]
fn test_policies_bulk_policy_key_manipulation() -> Result<(), Box<dyn std::error::Error>> {
    let policy1_name = "test_policies_bulk_policy_key_manipulation-1";
    let policy2_name = "test_policies_bulk_policy_key_manipulation-2";

    run_succeeds([
        "policies",
        "declare",
        "--name",
        policy1_name,
        "--pattern",
        "foo-.*",
        "--apply-to",
        "queues",
        "--priority",
        "123",
        "--definition",
        "{\"max-length\": 20, \"max-length-bytes\": 99999999}",
    ]);
    run_succeeds([
        "policies",
        "declare",
        "--name",
        policy2_name,
        "--pattern",
        "foo-.*",
        "--apply-to",
        "queues",
        "--priority",
        "123",
        "--definition",
        "{\"max-length\": 120, \"max-length-bytes\": 333333333}",
    ]);
    run_succeeds(["policies", "list"])
        .stdout(predicate::str::contains(policy1_name).and(predicate::str::contains("20")));
    run_succeeds(["policies", "list"])
        .stdout(predicate::str::contains(policy1_name).and(predicate::str::contains("333333333")));

    run_succeeds([
        "policies",
        "update_definitions_of_all_in",
        "--definition-key",
        "max-length",
        "--new-value",
        "272",
    ]);

    run_succeeds(["policies", "list"]).stdout(
        predicate::str::contains(policy1_name)
            .and(predicate::str::contains("272"))
            .and(predicate::str::contains("120").not()),
    );

    run_succeeds([
        "policies",
        "delete_definition_key_from_all_in",
        "--definition-key",
        "max-length",
    ]);

    run_succeeds(["policies", "list"])
        .stdout(predicate::str::contains(policy1_name).and(predicate::str::contains("333333333")));

    run_succeeds(["policies", "list"]).stdout(predicate::str::contains("272").not());

    run_succeeds(["policies", "delete", "--name", policy1_name]);
    run_succeeds(["policies", "delete", "--name", policy2_name]);
    run_succeeds(["policies", "list"]).stdout(
        predicate::str::contains(policy1_name)
            .not()
            .and(predicate::str::contains(policy2_name).not()),
    );

    Ok(())
}
