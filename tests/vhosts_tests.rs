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
fn test_list_vhosts() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "list_vhosts.1";
    delete_vhost(vh).expect("failed to delete a virtual host");

    run_succeeds(["declare", "vhost", "--name", vh]);
    run_succeeds(["list", "vhosts"])
        .stdout(predicate::str::contains("/").and(predicate::str::contains(vh)));

    delete_vhost(vh).expect("failed to delete a virtual host");
    run_succeeds(["list", "vhosts"])
        .stdout(predicate::str::contains("/").and(predicate::str::contains(vh).not()));

    Ok(())
}

#[test]
fn test_vhosts_list() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "list_vhosts.2";
    delete_vhost(vh).expect("failed to delete a virtual host");

    run_succeeds(["vhosts", "declare", "--name", vh]);
    run_succeeds(["vhosts", "list"])
        .stdout(predicate::str::contains("/").and(predicate::str::contains(vh)));

    delete_vhost(vh).expect("failed to delete a virtual host");
    run_succeeds(["vhosts", "list"])
        .stdout(predicate::str::contains("/").and(predicate::str::contains(vh).not()));

    Ok(())
}

#[test]
fn test_vhosts_create() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "vhosts.create.1";
    delete_vhost(vh).expect("failed to delete a virtual host");

    run_succeeds([
        "vhosts",
        "declare",
        "--name",
        vh,
        "--default-queue-type",
        "quorum",
        "--description",
        "just a test vhost",
        "--tracing",
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_vhosts_delete() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "vhosts.delete.1";
    run_succeeds(["vhosts", "delete", "--name", vh, "--idempotently"]);

    run_succeeds(["vhosts", "declare", "--name", vh]);

    run_succeeds(["vhosts", "delete", "--name", vh]);

    run_succeeds(["vhosts", "delete", "--name", vh, "--idempotently"]);

    Ok(())
}
