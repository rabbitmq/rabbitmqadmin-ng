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

use assert_cmd::Command;
use predicates::prelude::*;

mod common;
use crate::common::*;

#[test]
fn list_queues() -> Result<(), Box<dyn std::error::Error>> {
    // declare vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["declare", "vhost", "--name", "queue_vhost_1"]);
    cmd.assert().success();

    // declare vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["declare", "vhost", "--name", "queue_vhost_2"]);
    cmd.assert().success();

    // declare new queue in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("queue_vhost_1")
        .arg("declare")
        .arg("queue")
        .arg("--name")
        .arg("new_queue1")
        .arg("--type")
        .arg("classic");
    cmd.assert().success();

    // declare new queue in vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("queue_vhost_2")
        .arg("declare")
        .arg("queue")
        .arg("--name")
        .arg("new_queue2")
        .arg("--type")
        .arg("quorum");
    cmd.assert().success();

    await_queue_metric_emission();

    // list queues in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["-V", "queue_vhost_1", "list", "queues"]);
    cmd.assert().success().stdout(
        predicate::str::contains("new_queue1").and(predicate::str::contains("new_queue2").not()),
    );

    // delete the queues from vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("queue_vhost_1")
        .arg("delete")
        .arg("queue")
        .arg("--name")
        .arg("new_queue1");
    cmd.assert().success();

    // list queue in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V").arg("queue_vhost_1").arg("list").arg("queues");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("new_queue1").not());

    // delete vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "vhost", "--name", "queue_vhost_1"]);
    cmd.assert().success();

    // delete vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "vhost", "--name", "queue_vhost_2"]);
    cmd.assert().success();

    Ok(())
}
