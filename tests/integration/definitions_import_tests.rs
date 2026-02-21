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

use crate::test_helpers::delete_vhost;
use crate::test_helpers::run_succeeds;
use std::error::Error;
#[test]
fn test_import_cluster_definitions() -> Result<(), Box<dyn Error>> {
    let q = "queue_from_definitions";
    run_succeeds(["delete", "queue", "--name", q, "--idempotently"]);

    run_succeeds([
        "definitions",
        "import",
        "--file",
        "tests/fixtures/definitions/cluster.definitions.1.json",
    ]);

    run_succeeds(["delete", "queue", "--name", q, "--idempotently"]);

    Ok(())
}

#[test]
fn test_import_vhost_definitions() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.definitions_import.test1";

    delete_vhost(vh).expect("failed to delete a virtual host");
    run_succeeds(["declare", "vhost", "--name", vh]);

    run_succeeds([
        "--vhost",
        vh,
        "definitions",
        "import_into_vhost",
        "--file",
        "tests/fixtures/definitions/vhost.definitions.1.json",
    ]);

    run_succeeds(["delete", "vhost", "--name", vh, "--idempotently"]);

    Ok(())
}
