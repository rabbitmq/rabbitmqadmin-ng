// Copyright (C) 2023-2025 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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
use crate::test_helpers::*;

#[test]
fn test_messages() -> Result<(), Box<dyn std::error::Error>> {
    // declare a new queue
    let q = "publish_consume";
    run_succeeds(["declare", "queue", "--name", q, "--type", "classic"]);

    // publish a message
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

    // consume a message
    run_succeeds(["get", "messages", "--queue", q]).stdout(predicate::str::contains(payload));

    // delete the test queue
    run_succeeds(["delete", "queue", "--name", q]);

    Ok(())
}
