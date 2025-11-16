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
use std::error::Error;

mod test_helpers;
use crate::test_helpers::*;

#[test]
fn list_queues() -> Result<(), Box<dyn Error>> {
    let vh1 = "rabbitmqadmin.queue_vhost_1";
    let vh2 = "rabbitmqadmin.queue_vhost_2";
    let q1 = "new_queue1";
    let q2 = "new_queue2";

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    run_succeeds(["declare", "vhost", "--name", vh1]);

    run_succeeds(["declare", "vhost", "--name", vh2]);

    run_succeeds([
        "-V", vh1, "declare", "queue", "--name", q1, "--type", "classic",
    ]);

    run_succeeds([
        "-V", vh2, "declare", "queue", "--name", q2, "--type", "quorum",
    ]);

    await_queue_metric_emission();

    run_succeeds(["-V", vh1, "list", "queues"])
        .stdout(output_includes(q1).and(output_includes("new_queue2").not()));

    run_succeeds(["-V", vh1, "purge", "queue", "--name", q1]);

    run_succeeds(["-V", vh1, "delete", "queue", "--name", q1]);

    run_succeeds(["-V", vh1, "list", "queues"]).stdout(output_includes(q1).not());

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn queues_lists() -> Result<(), Box<dyn Error>> {
    let vh1 = "rabbitmqadmin.queue_vhost_3";
    let vh2 = "rabbitmqadmin.queue_vhost_4";
    let q1 = "new_queue1";
    let q2 = "new_queue2";

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    run_succeeds(["vhosts", "declare", "--name", vh1]);

    run_succeeds(["vhosts", "declare", "--name", vh2]);

    run_succeeds([
        "-V", vh1, "queues", "declare", "--name", q1, "--type", "classic",
    ]);

    run_succeeds([
        "-V", vh2, "queues", "declare", "--name", q2, "--type", "quorum",
    ]);

    await_queue_metric_emission();

    run_succeeds(["-V", vh1, "queues", "list"])
        .stdout(output_includes(q1).and(output_includes("new_queue2").not()));

    run_succeeds(["-V", vh1, "queues", "purge", "--name", q1]);

    run_succeeds(["-V", vh1, "queues", "delete", "--name", q1]);

    run_succeeds(["-V", vh1, "queues", "list"]).stdout(output_includes(q1).not());

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_queues_delete_idempotently() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.queues.test1";
    let q = "test_queue_delete_idempotently";

    delete_vhost(vh).expect("failed to delete a virtual host");
    run_succeeds(["declare", "vhost", "--name", vh]);

    run_succeeds(["-V", vh, "queues", "delete", "--name", q, "--idempotently"]);

    run_succeeds([
        "-V", vh, "declare", "queue", "--name", q, "--type", "classic",
    ]);

    run_succeeds(["-V", vh, "queues", "delete", "--name", q]);

    run_succeeds(["-V", vh, "queues", "delete", "--name", q, "--idempotently"]);

    run_succeeds([
        "declare", "queue", "-V", vh, "--name", q, "--type", "classic",
    ]);
    run_succeeds(["delete", "queue", "-V", vh, "--name", q]);
    run_succeeds(["delete", "queue", "-V", vh, "--name", q, "--idempotently"]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}
