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

mod test_helpers;
use crate::test_helpers::*;

#[test]
fn test_list_bindings() -> Result<(), Box<dyn std::error::Error>> {
    // declare vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["declare", "vhost", "--name", "bindings_vhost_1"]);
    cmd.assert().success();

    // declare vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["declare", "vhost", "--name", "bindings_vhost_2"]);
    cmd.assert().success();

    // declare new queue in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("bindings_vhost_1")
        .arg("declare")
        .arg("queue")
        .arg("--name")
        .arg("new_queue_1")
        .arg("--type")
        .arg("classic");
    cmd.assert().success();

    // declare new queue in vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("bindings_vhost_2")
        .arg("declare")
        .arg("queue")
        .arg("--name")
        .arg("new_queue_2")
        .arg("--type")
        .arg("quorum");
    cmd.assert().success();

    // declare exchange -> queue binding
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("bindings_vhost_1")
        .arg("declare")
        .arg("binding")
        .arg("--source")
        .arg("amq.direct")
        .arg("--destination_type")
        .arg("queue")
        .arg("--destination")
        .arg("new_queue_1")
        .arg("--routing_key")
        .arg("routing_key_queue");
    cmd.assert().success();

    // declare exchange -> exchange binding
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("bindings_vhost_1")
        .arg("declare")
        .arg("binding")
        .arg("--source")
        .arg("amq.direct")
        .arg("--destination_type")
        .arg("exchange")
        .arg("--destination")
        .arg("amq.topic")
        .arg("--routing_key")
        .arg("routing_key_exchange");
    cmd.assert().success();
    await_queue_metric_emission();

    // list bindings in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["-V", "bindings_vhost_1", "list", "bindings"]);
    cmd.assert().success().stdout(
        predicate::str::contains("new_queue_1")
            .and(predicate::str::contains("routing_key_queue"))
            .and(predicate::str::contains("routing_key_exchange")),
    );

    // delete the queues from vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("bindings_vhost_1")
        .arg("delete")
        .arg("queue")
        .arg("--name")
        .arg("new_queue_1");
    cmd.assert().success();

    // this routing_key should not longer be present
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["-V", "bindings_vhost_1", "list", "bindings"]);
    cmd.assert().success().stdout(
        predicate::str::contains("new_queue_1")
            .not()
            .and(predicate::str::contains("routing_key_queue"))
            .not()
            .and(predicate::str::contains("routing_key_exchange")),
    );

    // delete vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "vhost", "--name", "bindings_vhost_1"]);
    cmd.assert().success();

    // delete vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "vhost", "--name", "bindings_vhost_2"]);
    cmd.assert().success();
    Ok(())
}
