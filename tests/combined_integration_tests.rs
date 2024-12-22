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
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

mod test_helpers;

#[test]
fn combined_integration_test1() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "combined_integration_test1";
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.args([
        "--config",
        "tests/fixtures/config_Files/config_file1.conf",
        "--node",
        "node_a",
        "declare",
        "vhost",
        "--name",
        vh,
    ])
    .assert()
    .success();

    test_helpers::delete_vhost(vh)
}

#[test]
fn combined_integration_test2() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "combined_integration_test2";
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    // Uses a node alias that does not exist in the file
    cmd.args([
        "--config",
        "tests/fixtures/config_Files/config_file1.conf",
        "--node",
        "n0n_ex1stent_nod3",
        "declare",
        "vhost",
        "--name",
        vh,
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "was not found in the configuration file",
    ));

    test_helpers::delete_vhost(vh)
}

#[test]
fn combined_integration_test3() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "combined_integration_test3";
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    // Uses a node alias that does not exist in the file
    cmd.args([
        "--config",
        "tests/fixtures/config_Files/non_exis7ent_c0nfig_f1le.conf",
        "declare",
        "vhost",
        "--name",
        vh,
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("does not exist"));

    test_helpers::delete_vhost(vh)
}
