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

use crate::test_helpers::*;
use predicates::prelude::*;
use std::error::Error;

#[test]
fn test_list_conflicting_in_no_conflicts() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.policies_conflicting.1";
    run_succeeds(["delete", "vhost", "--name", vh, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh]);

    let policy1 = "test_list_conflicting_in_no_conflicts_1";
    let policy2 = "test_list_conflicting_in_no_conflicts_2";

    run_succeeds([
        "--vhost",
        vh,
        "policies",
        "declare",
        "--name",
        policy1,
        "--pattern",
        "^a-.*",
        "--apply-to",
        "queues",
        "--priority",
        "10",
        "--definition",
        "{\"max-length\": 100}",
    ]);

    run_succeeds([
        "--vhost",
        vh,
        "policies",
        "declare",
        "--name",
        policy2,
        "--pattern",
        "^b-.*",
        "--apply-to",
        "queues",
        "--priority",
        "20",
        "--definition",
        "{\"max-length\": 200}",
    ]);

    run_succeeds(["--vhost", vh, "policies", "list_conflicting_in"])
        .stdout(output_includes(policy1).not())
        .stdout(output_includes(policy2).not());

    run_succeeds(["--vhost", vh, "policies", "delete", "--name", policy1]);
    run_succeeds(["--vhost", vh, "policies", "delete", "--name", policy2]);
    run_succeeds(["delete", "vhost", "--name", vh]);

    Ok(())
}

#[test]
fn test_list_conflicting_in_with_conflicts() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.policies_conflicting.2";
    run_succeeds(["delete", "vhost", "--name", vh, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh]);

    let policy1 = "test_list_conflicting_in_with_conflicts_1";
    let policy2 = "test_list_conflicting_in_with_conflicts_2";
    let policy3 = "test_list_conflicting_in_with_conflicts_3";

    run_succeeds([
        "--vhost",
        vh,
        "policies",
        "declare",
        "--name",
        policy1,
        "--pattern",
        "^a-.*",
        "--apply-to",
        "queues",
        "--priority",
        "50",
        "--definition",
        "{\"max-length\": 100}",
    ]);

    run_succeeds([
        "--vhost",
        vh,
        "policies",
        "declare",
        "--name",
        policy2,
        "--pattern",
        "^b-.*",
        "--apply-to",
        "queues",
        "--priority",
        "50",
        "--definition",
        "{\"max-length\": 200}",
    ]);

    run_succeeds([
        "--vhost",
        vh,
        "policies",
        "declare",
        "--name",
        policy3,
        "--pattern",
        "^c-.*",
        "--apply-to",
        "queues",
        "--priority",
        "99",
        "--definition",
        "{\"max-length\": 300}",
    ]);

    run_succeeds(["--vhost", vh, "policies", "list_conflicting_in"])
        .stdout(output_includes(policy1))
        .stdout(output_includes(policy2))
        .stdout(output_includes(policy3).not());

    run_succeeds(["--vhost", vh, "policies", "delete", "--name", policy1]);
    run_succeeds(["--vhost", vh, "policies", "delete", "--name", policy2]);
    run_succeeds(["--vhost", vh, "policies", "delete", "--name", policy3]);
    run_succeeds(["delete", "vhost", "--name", vh]);

    Ok(())
}

#[test]
fn test_list_conflicting_across_vhosts() -> Result<(), Box<dyn Error>> {
    let vh1 = "rabbitmqadmin.policies_conflicting.3a";
    let vh2 = "rabbitmqadmin.policies_conflicting.3b";

    run_succeeds(["delete", "vhost", "--name", vh1, "--idempotently"]);
    run_succeeds(["delete", "vhost", "--name", vh2, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh1]);
    run_succeeds(["declare", "vhost", "--name", vh2]);

    let policy1 = "test_list_conflicting_across_vhosts_1";
    let policy2 = "test_list_conflicting_across_vhosts_2";
    let policy3 = "test_list_conflicting_across_vhosts_3";

    run_succeeds([
        "--vhost",
        vh1,
        "policies",
        "declare",
        "--name",
        policy1,
        "--pattern",
        "^a-.*",
        "--apply-to",
        "queues",
        "--priority",
        "75",
        "--definition",
        "{\"max-length\": 100}",
    ]);

    run_succeeds([
        "--vhost",
        vh1,
        "policies",
        "declare",
        "--name",
        policy2,
        "--pattern",
        "^b-.*",
        "--apply-to",
        "queues",
        "--priority",
        "75",
        "--definition",
        "{\"max-length\": 200}",
    ]);

    run_succeeds([
        "--vhost",
        vh2,
        "policies",
        "declare",
        "--name",
        policy3,
        "--pattern",
        "^c-.*",
        "--apply-to",
        "queues",
        "--priority",
        "75",
        "--definition",
        "{\"max-length\": 300}",
    ]);

    // list_conflicting shows all policies with conflicts across all vhosts
    // policy1 and policy2 conflict (same vhost, same priority)
    // policy3 does not conflict (different vhost)
    run_succeeds(["policies", "list_conflicting"])
        .stdout(output_includes(policy1))
        .stdout(output_includes(policy2))
        .stdout(output_includes(policy3).not());

    // list_conflicting_in only shows conflicts within specified vhost
    run_succeeds(["--vhost", vh1, "policies", "list_conflicting_in"])
        .stdout(output_includes(policy1))
        .stdout(output_includes(policy2));

    run_succeeds(["--vhost", vh2, "policies", "list_conflicting_in"])
        .stdout(output_includes(policy3).not());

    run_succeeds(["--vhost", vh1, "policies", "delete", "--name", policy1]);
    run_succeeds(["--vhost", vh1, "policies", "delete", "--name", policy2]);
    run_succeeds(["--vhost", vh2, "policies", "delete", "--name", policy3]);
    run_succeeds(["delete", "vhost", "--name", vh1]);
    run_succeeds(["delete", "vhost", "--name", vh2]);

    Ok(())
}

#[test]
fn test_list_conflicting_multiple_conflict_groups() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.policies_conflicting.4";
    run_succeeds(["delete", "vhost", "--name", vh, "--idempotently"]);
    run_succeeds(["declare", "vhost", "--name", vh]);

    let policy1 = "test_list_conflicting_multi_1";
    let policy2 = "test_list_conflicting_multi_2";
    let policy3 = "test_list_conflicting_multi_3";
    let policy4 = "test_list_conflicting_multi_4";

    run_succeeds([
        "--vhost",
        vh,
        "policies",
        "declare",
        "--name",
        policy1,
        "--pattern",
        "^a-.*",
        "--apply-to",
        "queues",
        "--priority",
        "10",
        "--definition",
        "{\"max-length\": 100}",
    ]);

    run_succeeds([
        "--vhost",
        vh,
        "policies",
        "declare",
        "--name",
        policy2,
        "--pattern",
        "^b-.*",
        "--apply-to",
        "queues",
        "--priority",
        "10",
        "--definition",
        "{\"max-length\": 200}",
    ]);

    run_succeeds([
        "--vhost",
        vh,
        "policies",
        "declare",
        "--name",
        policy3,
        "--pattern",
        "^c-.*",
        "--apply-to",
        "queues",
        "--priority",
        "20",
        "--definition",
        "{\"max-length\": 300}",
    ]);

    run_succeeds([
        "--vhost",
        vh,
        "policies",
        "declare",
        "--name",
        policy4,
        "--pattern",
        "^d-.*",
        "--apply-to",
        "queues",
        "--priority",
        "20",
        "--definition",
        "{\"max-length\": 400}",
    ]);

    run_succeeds(["policies", "list_conflicting"])
        .stdout(output_includes(policy1))
        .stdout(output_includes(policy2))
        .stdout(output_includes(policy3))
        .stdout(output_includes(policy4));

    run_succeeds(["--vhost", vh, "policies", "list_conflicting_in"])
        .stdout(output_includes(policy1))
        .stdout(output_includes(policy2))
        .stdout(output_includes(policy3))
        .stdout(output_includes(policy4));

    run_succeeds(["--vhost", vh, "policies", "delete", "--name", policy1]);
    run_succeeds(["--vhost", vh, "policies", "delete", "--name", policy2]);
    run_succeeds(["--vhost", vh, "policies", "delete", "--name", policy3]);
    run_succeeds(["--vhost", vh, "policies", "delete", "--name", policy4]);
    run_succeeds(["delete", "vhost", "--name", vh]);

    Ok(())
}
