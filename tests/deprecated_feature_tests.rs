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
fn test_list_all_deprecated_features() -> Result<(), Box<dyn std::error::Error>> {
    run_succeeds(["deprecated_features", "list"]).stdout(predicate::str::contains("ram_node_type"));

    Ok(())
}

#[test]
fn test_list_deprecated_features_in_use() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_list_deprecated_features_in_use";
    let q = "test_list_deprecated_features_in_use.cq.transient.1";

    delete_vhost(vh).expect("failed to delete a virtual host");

    // there are no deprecated features in use at this point
    run_succeeds(["deprecated_features", "list_used"])
        .stdout(predicate::str::contains("transient_nonexcl_queues").not());

    run_succeeds(["declare", "vhost", "--name", vh]);
    run_succeeds([
        "-V",
        vh,
        "declare",
        "queue",
        "--name",
        q,
        "--type",
        "classic",
        "--durable",
        "false",
    ]);

    await_queue_metric_emission();

    // now there is: a non-exclusive transient queue
    run_succeeds(["list", "deprecated_features_in_use"])
        .stdout(predicate::str::contains("transient_nonexcl_queues"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_list_all_deprecated_features_via_alias() -> Result<(), Box<dyn std::error::Error>> {
    run_succeeds(["list", "deprecated_features"]).stdout(predicate::str::contains("ram_node_type"));

    Ok(())
}

#[test]
fn test_list_deprecated_features_in_use_via_alias() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_list_deprecated_features_in_use_via_alias";
    let q = "test_list_deprecated_features_in_use_via_alias.cq.transient.1";

    delete_vhost(vh).expect("failed to delete a virtual host");

    // there are no deprecated features in use at this point
    run_succeeds(["list", "deprecated_features_in_use"])
        .stdout(predicate::str::contains("transient_nonexcl_queues").not());

    run_succeeds(["declare", "vhost", "--name", vh]);
    run_succeeds([
        "-V",
        vh,
        "declare",
        "queue",
        "--name",
        q,
        "--type",
        "classic",
        "--durable",
        "false",
    ]);

    await_queue_metric_emission();

    // now there is: a non-exclusive transient queue
    run_succeeds(["list", "deprecated_features_in_use"])
        .stdout(predicate::str::contains("transient_nonexcl_queues"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}
