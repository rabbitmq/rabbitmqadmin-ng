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

use predicates::prelude::*;
use std::error::Error;
mod test_helpers;
use crate::test_helpers::*;

#[test]
fn test_list_operator_policies() -> Result<(), Box<dyn Error>> {
    let policy_name = "test_list_operator_policies";

    run_succeeds([
        "operator_policies",
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
        "{\"max-length\": 12345}",
    ]);

    run_succeeds(["operator_policies", "list"])
        .stdout(output_includes(policy_name).and(output_includes("12345")));
    run_succeeds(["delete", "operator_policy", "--name", policy_name]);
    run_succeeds(["operator_policies", "list"]).stdout(output_includes(policy_name).not());

    Ok(())
}

#[test]
fn test_operator_policies() -> Result<(), Box<dyn Error>> {
    let operator_policy_name = "test_operator_policies.1";

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

    run_succeeds(["list", "operator_policies"])
        .stdout(output_includes(operator_policy_name).and(output_includes("op-foo")));
    run_succeeds(["delete", "operator_policy", "--name", operator_policy_name]);
    run_succeeds(["list", "operator_policies"]).stdout(output_includes(operator_policy_name).not());

    Ok(())
}

#[test]
fn test_operator_policies_declare_list_and_delete() -> Result<(), Box<dyn Error>> {
    let policy_name = "test_operator_policies_declare_list_and_delete";

    run_succeeds([
        "operator_policies",
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

    run_succeeds(["operator_policies", "list"])
        .stdout(output_includes(policy_name).and(output_includes("20")));
    run_succeeds(["operator_policies", "delete", "--name", policy_name]);
    run_succeeds(["operator_policies", "list"]).stdout(output_includes(policy_name).not());

    Ok(())
}

#[test]
fn test_operator_policies_in() -> Result<(), Box<dyn Error>> {
    let vh1 = "rabbitmqadmin.test_operator_policies_in.1";
    run_succeeds(["delete", "vhost", "--name", vh1, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh1]);

    let vh2 = "rabbitmqadmin.test_operator_policies_in.2";
    run_succeeds(["delete", "vhost", "--name", vh2, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh2]);

    let policy_name = "test_operator_policies_in";
    run_succeeds([
        "--vhost",
        vh1,
        "operator_policies",
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

    run_succeeds(["--vhost", vh1, "operator_policies", "list_in"])
        .stdout(output_includes(policy_name).and(output_includes("98")));
    run_succeeds(["--vhost", vh2, "operator_policies", "list_in"])
        .stdout(output_includes(policy_name).not());
    run_succeeds([
        "--vhost",
        vh1,
        "operator_policies",
        "delete",
        "--name",
        policy_name,
    ]);
    run_succeeds(["--vhost", vh1, "operator_policies", "list_in"])
        .stdout(output_includes(policy_name).not());

    run_succeeds(["delete", "vhost", "--name", vh1]);
    run_succeeds(["delete", "vhost", "--name", vh2]);

    Ok(())
}

#[test]
fn test_operator_policies_in_with_entity_type() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.operator_policies.test1";
    run_succeeds(["delete", "vhost", "--name", vh, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh]);

    let policy_name = "test_operator_policies_in_with_entity_type";
    run_succeeds([
        "--vhost",
        vh,
        "operator_policies",
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

    run_succeeds([
        "--vhost",
        vh,
        "operator_policies",
        "list_in",
        "--apply-to",
        "queues",
    ])
    .stdout(output_includes(policy_name).and(output_includes("98")));
    run_succeeds([
        "--vhost",
        vh,
        "operator_policies",
        "list_in",
        "--apply-to",
        "exchanges",
    ])
    .stdout(output_includes(policy_name).not());
    run_succeeds([
        "--vhost",
        vh,
        "operator_policies",
        "list_in",
        "--apply-to",
        "streams",
    ])
    .stdout(output_includes(policy_name).not());
    run_succeeds([
        "--vhost",
        "/",
        "operator_policies",
        "list_in",
        "--apply-to",
        "queues",
    ])
    .stdout(output_includes(policy_name).not());
    run_succeeds([
        "--vhost",
        vh,
        "operator_policies",
        "delete",
        "--name",
        policy_name,
    ]);
    run_succeeds(["--vhost", vh, "operator_policies", "list_in"])
        .stdout(output_includes(policy_name).not());

    run_succeeds(["delete", "vhost", "--name", vh]);

    Ok(())
}

#[test]
fn test_operator_policies_matching_objects() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.operator_policies.test2";

    run_succeeds(["delete", "vhost", "--name", vh, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh]);

    let policy_name = "rabbitmqadmin.operator_policies.11";
    run_succeeds([
        "--vhost",
        vh,
        "operator_policies",
        "declare",
        "--name",
        policy_name,
        "--pattern",
        "^q-.*",
        "--apply-to",
        "queues",
        "--priority",
        "47",
        "--definition",
        "{\"max-length\": 20}",
    ]);

    run_succeeds([
        "--vhost",
        vh,
        "operator_policies",
        "list_matching_object",
        "--name",
        "q-abc",
        "--type",
        "queues",
    ])
    .stdout(output_includes(policy_name).and(output_includes("20")));
    run_succeeds([
        "--vhost",
        vh,
        "operator_policies",
        "list_matching_object",
        "--name",
        "q-abc",
        "--type",
        "exchanges",
    ])
    .stdout(output_includes(policy_name).not());

    run_succeeds(["delete", "vhost", "--name", vh, "--idempotently"]);

    Ok(())
}

#[test]
fn test_operator_policies_declare_list_update_definition_and_delete() -> Result<(), Box<dyn Error>>
{
    let policy_name = "test_operator_policies_declare_list_update_definition_and_delete";

    run_succeeds([
        "operator_policies",
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
    run_succeeds(["operator_policies", "list"])
        .stdout(output_includes(policy_name).and(output_includes("20")));

    run_succeeds([
        "operator_policies",
        "update_definition",
        "--name",
        policy_name,
        "--definition-key",
        "max-length",
        "--new-value",
        "131",
    ]);

    run_succeeds(["operator_policies", "list"])
        .stdout(output_includes(policy_name).and(output_includes("131")));

    run_succeeds(["operator_policies", "delete", "--name", policy_name]);
    run_succeeds(["operator_policies", "list"]).stdout(output_includes(policy_name).not());

    Ok(())
}

#[test]
fn test_operator_policies_individual_policy_key_manipulation() -> Result<(), Box<dyn Error>> {
    let policy_name = "test_operator_policies_individual_policy_key_manipulation";

    run_succeeds([
        "operator_policies",
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
        "{\"max-length\": 20, \"max-length-bytes\": 128372836172}",
    ]);
    run_succeeds(["operator_policies", "list"])
        .stdout(output_includes(policy_name).and(output_includes("20")));

    run_succeeds([
        "operator_policies",
        "update_definition",
        "--name",
        policy_name,
        "--definition-key",
        "max-length",
        "--new-value",
        "131",
    ]);

    run_succeeds(["operator_policies", "list"])
        .stdout(output_includes(policy_name).and(output_includes("131")));

    run_succeeds([
        "operator_policies",
        "delete_definition_keys",
        "--name",
        policy_name,
        "--definition-keys",
        "max-length,abc,def",
    ]);

    run_succeeds(["operator_policies", "list"])
        .stdout(output_includes(policy_name).and(output_includes("128372836172")));

    run_succeeds(["operator_policies", "list"]).stdout(output_includes("131").not());

    run_succeeds(["operator_policies", "delete", "--name", policy_name]);
    run_succeeds(["operator_policies", "list"]).stdout(output_includes(policy_name).not());

    Ok(())
}

#[test]
fn test_operator_policies_bulk_policy_keys_manipulation() -> Result<(), Box<dyn Error>> {
    let vh1 = "rabbitmqadmin.test_operator_policies_bulk_policy_keys_manipulation.1";
    let vh2 = "rabbitmqadmin.test_operator_policies_bulk_policy_keys_manipulation.2";

    run_succeeds(["delete", "vhost", "--name", vh1, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh1]);
    run_succeeds(["delete", "vhost", "--name", vh2, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh2]);

    let policy1_name = "test_operator_policies_bulk_policy_keys_manipulation-1";
    let policy2_name = "test_operator_policies_bulk_policy_keys_manipulation-2";

    run_succeeds([
        "--vhost",
        vh1,
        "operator_policies",
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
        "{\"max-length\": 20, \"max-length-bytes\": 467467467467}",
    ]);
    run_succeeds([
        "--vhost",
        vh2,
        "operator_policies",
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
    run_succeeds(["operator_policies", "list"])
        .stdout(output_includes(policy1_name).and(output_includes("20")));
    run_succeeds(["operator_policies", "list"])
        .stdout(output_includes(policy1_name).and(output_includes("333333333")));

    run_succeeds([
        "--vhost",
        vh2,
        "operator_policies",
        "update_definitions_of_all_in",
        "--definition-key",
        "max-length",
        "--new-value",
        "272",
    ]);

    run_succeeds(["operator_policies", "list"]).stdout(
        output_includes(policy1_name)
            .and(output_includes("272"))
            .and(output_includes("120").not()),
    );

    run_succeeds([
        "--vhost",
        vh1,
        "operator_policies",
        "delete_definition_keys_from_all_in",
        "--definition-keys",
        "max-length,other-key",
    ]);

    run_succeeds([
        "--vhost",
        vh2,
        "operator_policies",
        "delete_definition_keys_from_all_in",
        "--definition-keys",
        "max-length",
    ]);

    run_succeeds(["operator_policies", "list"])
        .stdout(output_includes(policy1_name).and(output_includes("333333333")));

    run_succeeds(["operator_policies", "list"]).stdout(output_includes("272").not());

    run_succeeds([
        "--vhost",
        vh1,
        "operator_policies",
        "delete",
        "--name",
        policy1_name,
    ]);
    run_succeeds([
        "--vhost",
        vh2,
        "operator_policies",
        "delete",
        "--name",
        policy2_name,
    ]);
    run_succeeds(["operator_policies", "list"]).stdout(
        output_includes(policy1_name)
            .not()
            .and(output_includes(policy2_name).not()),
    );

    Ok(())
}

#[test]
fn test_operator_policies_patch_definition() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.operator_policies.test3";
    run_succeeds(["delete", "vhost", "--name", vh, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh]);

    let policy_name = "test_operator_policies_patch_definition.ad6f7d";

    run_succeeds([
        "--vhost",
        vh,
        "operator_policies",
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
        "{\"max-length\": 923, \"max-length-bytes\": 287237182378237}",
    ]);
    run_succeeds(["--vhost", vh, "operator_policies", "list"])
        .stdout(output_includes(policy_name).and(output_includes("923")));

    run_succeeds([
        "--vhost",
        vh,
        "operator_policies",
        "patch",
        "--name",
        policy_name,
        "--definition",
        "{\"max-length\": 875, \"max-length-bytes\": 12355242124}",
    ]);

    run_succeeds(["operator_policies", "list"]).stdout(
        output_includes(policy_name)
            .and(output_includes("12355242124"))
            .and(output_includes("875")),
    );

    run_succeeds(["operator_policies", "list"]).stdout(output_includes("287237182378237").not());

    run_succeeds([
        "--vhost",
        vh,
        "operator_policies",
        "delete",
        "--name",
        policy_name,
    ]);
    run_succeeds(["operator_policies", "list"]).stdout(output_includes(policy_name).not());

    run_succeeds(["delete", "vhost", "--name", vh, "--idempotently"]);

    Ok(())
}
