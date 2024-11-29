use std::process::Command;
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

#[test]
fn test_list_permissions() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.args([
        "declare",
        "user",
        "--name",
        "user_with_permissions",
        "--password",
        "pa$$w0rd",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args([
        "declare",
        "permissions",
        "--user",
        "user_with_permissions",
        "--configure",
        "foo",
        "--read",
        "bar",
        "--write",
        "baz",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "permissions"]);
    cmd.assert().success().stdout(
        predicate::str::contains("foo")
            .and(predicate::str::contains("bar"))
            .and(predicate::str::contains("baz")),
    );

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "permissions", "--user", "user_with_permissions"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "permissions"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("user_with_permissions").not());

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "user", "--name", "user_with_permissions"]);
    cmd.assert().success();
    Ok(())
}
