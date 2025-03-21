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
use crate::test_helpers::delete_vhost;
use test_helpers::run_succeeds;

#[test]
fn test_export_cluster_wide_definitions() -> Result<(), Box<dyn std::error::Error>> {
    run_succeeds(["definitions", "export"]).stdout(predicate::str::contains("guest"));

    Ok(())
}

#[test]
fn test_export_vhost_definitions() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_export_vhost_definitions.1";
    delete_vhost(vh).expect("failed to delete a virtual host");
    run_succeeds(["declare", "vhost", "--name", vh]);

    let q = "qq.test_export_vhost_definitions.1";
    run_succeeds([
        "-V", vh, "declare", "queue", "--name", q, "--type", "quorum",
    ]);

    run_succeeds(["--vhost", vh, "definitions", "export_from_vhost"])
        .stdout(predicate::str::contains(q));
    run_succeeds(["--vhost", "/", "definitions", "export_from_vhost"])
        .stdout(predicate::str::contains(q).not());

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_export_cluster_wide_definitions_with_transformations_case1()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_export_cluster_definitions.transformations.1";
    delete_vhost(vh).expect("failed to delete a virtual host");
    run_succeeds(["declare", "vhost", "--name", vh]);

    let p1 = "test_export_cluster_definitions.1";
    run_succeeds([
        "--vhost",
        vh,
        "declare",
        "policy",
        "--name",
        p1,
        "--pattern",
        "^matching\\..+",
        "--apply-to",
        "classic_queues",
        "--priority",
        "10",
        "--definition",
        "{\"max-length\": 10}",
    ]);

    let q = "qq.test_export_cluster_definitions.1";
    run_succeeds([
        "-V", vh, "declare", "queue", "--name", q, "--type", "quorum",
    ]);

    run_succeeds(["--vhost", vh, "definitions", "export"]).stdout(predicate::str::contains(p1));
    // These two cannot be tested on 4.x: empty definitions will be rejected
    // by validation and CMQ keys are no longer recognized as known/valid.
    // But at least we can test the code path this way.
    run_succeeds([
        "--vhost",
        vh,
        "definitions",
        "export",
        "--transformations",
        "strip_cmq_keys_from_policies,drop_empty_policies",
    ])
    .stdout(predicate::str::contains(p1));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}
