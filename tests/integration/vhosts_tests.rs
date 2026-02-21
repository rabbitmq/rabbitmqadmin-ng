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

use predicates::prelude::*;
use std::error::Error;

use crate::skip_if_rabbitmq_version_below;
use crate::test_helpers::*;

#[test]
fn test_list_vhosts() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.vhosts.test1";
    delete_vhost(vh).expect("failed to delete a virtual host");

    run_succeeds(["declare", "vhost", "--name", vh]);
    run_succeeds(["list", "vhosts"]).stdout(output_includes("/").and(output_includes(vh)));

    delete_vhost(vh).expect("failed to delete a virtual host");
    run_succeeds(["list", "vhosts"]).stdout(output_includes("/").and(output_includes(vh).not()));

    Ok(())
}

#[test]
fn test_vhosts_list() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.vhosts.test2";
    delete_vhost(vh).expect("failed to delete a virtual host");

    run_succeeds(["vhosts", "declare", "--name", vh]);
    run_succeeds(["vhosts", "list"]).stdout(output_includes("/").and(output_includes(vh)));

    delete_vhost(vh).expect("failed to delete a virtual host");
    run_succeeds(["vhosts", "list"]).stdout(output_includes("/").and(output_includes(vh).not()));

    Ok(())
}

#[test]
fn test_vhosts_create() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.vhosts.test3";
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
fn test_vhosts_delete() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.vhosts.test4";
    run_succeeds(["vhosts", "delete", "--name", vh, "--idempotently"]);

    run_succeeds(["vhosts", "declare", "--name", vh]);

    run_succeeds(["vhosts", "delete", "--name", vh]);

    run_succeeds(["vhosts", "delete", "--name", vh, "--idempotently"]);

    Ok(())
}

#[test]
fn test_vhosts_enable_deletion_protection() -> Result<(), Box<dyn Error>> {
    skip_if_rabbitmq_version_below!(4, 2, 0);

    let vh = "rabbitmqadmin.vhosts.test-deletion-protection-enable";
    run_succeeds(["vhosts", "delete", "--name", vh, "--idempotently"]);

    run_succeeds(["vhosts", "declare", "--name", vh]);

    run_succeeds(["vhosts", "enable_deletion_protection", "--name", vh]);

    run_succeeds(["vhosts", "disable_deletion_protection", "--name", vh]);
    run_succeeds(["vhosts", "delete", "--name", vh]);

    Ok(())
}

#[test]
fn test_vhosts_disable_deletion_protection() -> Result<(), Box<dyn Error>> {
    skip_if_rabbitmq_version_below!(4, 2, 0);

    let vh = "rabbitmqadmin.vhosts.test-deletion-protection-disable";
    run_succeeds(["vhosts", "delete", "--name", vh, "--idempotently"]);

    run_succeeds(["vhosts", "declare", "--name", vh]);

    run_succeeds(["vhosts", "enable_deletion_protection", "--name", vh]);
    run_succeeds(["vhosts", "disable_deletion_protection", "--name", vh]);

    run_succeeds(["vhosts", "delete", "--name", vh]);

    Ok(())
}

#[test]
fn test_vhosts_protected_vhost_cannot_be_deleted() -> Result<(), Box<dyn Error>> {
    skip_if_rabbitmq_version_below!(4, 2, 0);

    let vh = "rabbitmqadmin.vhosts.test-protected-cannot-delete";
    run_succeeds(["vhosts", "delete", "--name", vh, "--idempotently"]);

    run_succeeds(["vhosts", "declare", "--name", vh]);
    run_succeeds(["vhosts", "enable_deletion_protection", "--name", vh]);

    run_fails(["vhosts", "delete", "--name", vh]);

    run_succeeds(["vhosts", "disable_deletion_protection", "--name", vh]);
    run_succeeds(["vhosts", "delete", "--name", vh]);

    Ok(())
}
