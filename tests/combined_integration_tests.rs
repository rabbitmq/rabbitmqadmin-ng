// Copyright (C) 2023-2024 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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
use test_helpers::{run_fails, run_succeeds};

#[test]
fn combined_integration_test1() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "combined_integration_test1";
    run_succeeds([
        "--config",
        "tests/fixtures/config_Files/config_file1.conf",
        "--node",
        "node_a",
        "declare",
        "vhost",
        "--name",
        vh,
    ]);

    test_helpers::delete_vhost(vh)
}

#[test]
fn combined_integration_test2() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "combined_integration_test2";

    // Uses a node alias that does not exist in the file
    run_fails([
        "--config",
        "tests/fixtures/config_Files/config_file1.conf",
        "--node",
        "n0n_ex1stent_nod3",
        "declare",
        "vhost",
        "--name",
        vh,
    ])
    .stderr(predicate::str::contains(
        "was not found in the configuration file",
    ));

    test_helpers::delete_vhost(vh)
}

#[test]
fn combined_integration_test3() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "combined_integration_test3";

    // Uses a node alias that does not exist in the file
    run_fails([
        "--config",
        "tests/fixtures/config_Files/non_exis7ent_c0nfig_f1le.conf",
        "declare",
        "vhost",
        "--name",
        vh,
    ])
    .stderr(predicate::str::contains("does not exist"));

    test_helpers::delete_vhost(vh)
}

#[test]
fn combined_integration_test4() -> Result<(), Box<dyn std::error::Error>> {
    // This test uses administrative credentials to create a new user
    // and set up a topology using those new credentials
    let vh = "combined_integration_test4";
    let new_user = "user_from_combined_integration_test4";
    let new_pass = "p4$$w0rd_from_combined_integration_test4";
    let x = "fanout_combined_integration_test4";
    let q = "queue_from_combined_integration_test4";

    run_succeeds(["declare", "vhost", "--name", vh]);
    run_succeeds([
        "declare",
        "user",
        "--name",
        new_user,
        "--password",
        new_pass,
        "--tags",
        "administrator",
    ]);
    run_succeeds([
        "--vhost",
        vh,
        "declare",
        "permissions",
        "--user",
        new_user,
        "--configure",
        ".*",
        "--read",
        ".*",
        "--write",
        ".*",
    ]);
    run_succeeds([
        "--vhost",
        vh,
        "--username",
        new_user,
        "--password",
        new_pass,
        "declare",
        "exchange",
        "--name",
        x,
        "--type",
        "fanout",
        "--durable",
        "true",
        "--auto_delete",
        "false",
    ]);
    run_succeeds([
        "--vhost",
        vh,
        "--username",
        new_user,
        "--password",
        new_pass,
        "declare",
        "queue",
        "--name",
        q,
        "--type",
        "quorum",
        "--durable",
        "true",
        "--auto_delete",
        "false",
    ]);
    run_succeeds([
        "--vhost",
        vh,
        "--username",
        new_user,
        "--password",
        new_pass,
        "declare",
        "queue",
        "--name",
        q,
        "--type",
        "quorum",
        "--durable",
        "true",
        "--auto_delete",
        "false",
    ]);
    run_succeeds([
        "--vhost",
        vh,
        "--username",
        new_user,
        "--password",
        new_pass,
        "declare",
        "binding",
        "--source",
        x,
        "--destination_type",
        "queue",
        "--destination",
        q,
        "--routing_key",
        "rk",
    ]);

    // We don't have to clear this topology because the entire virtual host will be deleted
    // soon but this is an integration test, so let's do that
    run_succeeds([
        "--vhost",
        vh,
        "--username",
        new_user,
        "--password",
        new_pass,
        "delete",
        "binding",
        "--source",
        x,
        "--destination_type",
        "queue",
        "--destination",
        q,
        "--routing_key",
        "rk",
    ]);
    run_succeeds(["-V", vh, "delete", "exchange", "--name", x]);
    run_succeeds(["-V", vh, "delete", "queue", "--name", q]);

    test_helpers::delete_user(new_user).expect("failed to delete a user");
    test_helpers::delete_vhost(vh)
}