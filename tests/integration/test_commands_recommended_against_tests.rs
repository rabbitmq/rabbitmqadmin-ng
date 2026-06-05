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
use assert_cmd::prelude::*;
use std::error::Error;
use std::fs;
#[test]
fn test_messages() -> Result<(), Box<dyn Error>> {
    let q = "publish_consume";
    run_succeeds(["declare", "queue", "--name", q, "--type", "classic"]);

    let payload = "test_messages_1";
    run_succeeds([
        "publish",
        "message",
        "--routing-key",
        q,
        "--payload",
        payload,
        "--properties",
        "{\"timestamp\": 1234, \"message_id\": \"foo\"}",
    ]);

    run_succeeds(["get", "messages", "--queue", q]).stdout(output_includes(payload));

    run_succeeds(["delete", "queue", "--name", q]);

    Ok(())
}

#[test]
fn test_messages_payload_file() -> Result<(), Box<dyn Error>> {
    let q = "publish_consume_payload_file";
    run_succeeds(["declare", "queue", "--name", q, "--type", "classic"]);

    let file = "tests/fixtures/messages/message1.txt";
    run_succeeds([
        "publish",
        "message",
        "--routing-key",
        q,
        "--payload-file",
        file,
    ]);

    run_fails([
        "publish",
        "message",
        "--routing-key",
        q,
        "--payload-file",
        "nonexistent_file",
    ]);

    run_succeeds(["get", "messages", "--queue", q])
        .stdout(output_includes(&fs::read_to_string(file)?));

    run_succeeds(["delete", "queue", "--name", q]);

    Ok(())
}

#[test]
fn test_messages_payload_stdin() -> Result<(), Box<dyn Error>> {
    let q = "publish_consume_payload_stdin";
    run_succeeds(["declare", "queue", "--name", q, "--type", "classic"]);

    let file = "tests/fixtures/messages/message2.txt";

    std::process::Command::new(assert_cmd::cargo::cargo_bin!("rabbitmqadmin"))
        .args([
            "publish",
            "message",
            "--routing-key",
            q,
            "--payload-file",
            "-",
        ])
        .stdin(std::fs::File::open(file)?)
        .assert()
        .success();

    run_succeeds(["get", "messages", "--queue", q])
        .stdout(output_includes(&fs::read_to_string(file)?));

    run_succeeds(["delete", "queue", "--name", q]);

    Ok(())
}
