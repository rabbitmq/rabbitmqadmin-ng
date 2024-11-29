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
fn test_messages() -> Result<(), Box<dyn std::error::Error>> {
    // declare a new queue
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("declare")
        .arg("queue")
        .arg("--name")
        .arg("publish_consume")
        .arg("--type")
        .arg("classic");
    cmd.assert().success();

    // publish a message
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("publish")
        .arg("message")
        .arg("--routing-key")
        .arg("publish_consume")
        .arg("--payload")
        .arg("test_messages_1")
        .arg("--properties")
        .arg("{\"timestamp\": 1234, \"message_id\": \"foo\"}");
    cmd.assert().success();

    // consume a message
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("get")
        .arg("messages")
        .arg("--queue")
        .arg("publish_consume");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test_messages_1"));

    // delete the test queue
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("delete")
        .arg("queue")
        .arg("--name")
        .arg("publish_consume");
    cmd.assert().success();

    Ok(())
}
