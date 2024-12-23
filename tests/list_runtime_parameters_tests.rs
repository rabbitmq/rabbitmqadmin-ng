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
use crate::test_helpers::*;

#[test]
fn test_runtime_parameters() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "parameters_vhost_1";
    run_succeeds(["declare", "vhost", "--name", vh]);
    run_succeeds([
        "-V",
        vh,
        "declare",
        "parameter",
        "--component",
        "federation-upstream",
        "--name",
        "my-upstream",
        "--value",
        "{\"uri\":\"amqp://target.hostname\",\"expires\":3600000}",
    ]);

    run_succeeds([
        "-V",
        vh,
        "list",
        "parameters",
        "--component",
        "federation-upstream",
    ])
    .stdout(predicate::str::contains("my-upstream").and(predicate::str::contains("3600000")));

    run_succeeds([
        "-V",
        vh,
        "delete",
        "parameter",
        "--component",
        "federation-upstream",
        "--name",
        "my-upstream",
    ]);

    run_succeeds([
        "-V",
        vh,
        "list",
        "parameters",
        "--component",
        "federation-upstream",
    ])
    .stdout(predicate::str::contains("my-upstream").not());

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}
