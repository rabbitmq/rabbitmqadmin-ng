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
use assert_cmd::Command;

#[test]
fn test_list_policies() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.args([
        "declare",
        "policy",
        "--name",
        "test_policy",
        "--pattern",
        "foo-.*",
        "--apply-to",
        "queues",
        "--priority",
        "123",
        "--definition",
        "{\"max-length\": 12345}",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "policies"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test_policy").and(predicate::str::contains("12345")));

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "policy", "--name", "test_policy"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "policies"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test_policy").not());

    Ok(())
}

#[test]
fn test_operator_policies() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.args([
        "declare",
        "operator_policy",
        "--name",
        "test_operator_policy",
        "--pattern",
        "op-foo.*",
        "--apply-to",
        "queues",
        "--priority",
        "123",
        "--definition",
        "{\"max-length\": 12345}",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "operator_policies"]);
    cmd.assert().success().stdout(
        predicate::str::contains("test_operator_policy").and(predicate::str::contains("op-foo")),
    );

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args([
        "delete",
        "operator_policy",
        "--name",
        "test_operator_policy",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "operator_policies"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test_operator_policy").not());

    Ok(())
}