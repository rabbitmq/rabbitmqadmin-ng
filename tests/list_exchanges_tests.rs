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

#[test]
fn list_exchanges() -> Result<(), Box<dyn std::error::Error>> {
    // declare vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["declare", "vhost", "--name", "exchange_vhost_1"]);
    cmd.assert().success();

    // declare vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["declare", "vhost", "--name", "exchange_vhost_2"]);
    cmd.assert().success();

    // declare a new exchange in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("exchange_vhost_1")
        .arg("declare")
        .arg("exchange")
        .arg("--name")
        .arg("new_exchange1");
    cmd.assert().success();

    // declare a new exchange in vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("exchange_vhost_2")
        .arg("declare")
        .arg("exchange")
        .arg("--name")
        .arg("new_exchange2");
    cmd.assert().success();

    // list exchanges in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["-V", "exchange_vhost_1", "list", "exchanges"]);
    cmd.assert().success().stdout(
        predicate::str::contains("amq.direct")
            .and(predicate::str::contains("amq.fanout"))
            .and(predicate::str::contains("amq.headers"))
            .and(predicate::str::contains("amq.topic"))
            .and(predicate::str::contains("new_exchange1"))
            .and(predicate::str::contains("new_exchange2").not()),
    );

    // delete the exchanges from vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("exchange_vhost_1")
        .arg("delete")
        .arg("exchange")
        .arg("--name")
        .arg("new_exchange1");
    cmd.assert().success();

    // list exchange in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("exchange_vhost_1")
        .arg("list")
        .arg("exchanges");
    cmd.assert().success().stdout(
        predicate::str::contains("amq.direct")
            .and(predicate::str::contains("amq.fanout"))
            .and(predicate::str::contains("amq.headers"))
            .and(predicate::str::contains("amq.topic"))
            .and(predicate::str::contains("new_exchange1").not()),
    );

    // delete vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "vhost", "--name", "exchange_vhost_1"]);
    cmd.assert().success();

    // delete vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "vhost", "--name", "exchange_vhost_2"]);
    cmd.assert().success();

    Ok(())
}
