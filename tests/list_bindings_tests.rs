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
fn test_list_bindings() -> Result<(), Box<dyn std::error::Error>> {
    let vh1 = "bindings_vhost_1";
    let vh2 = "bindings_vhost_2";
    let q1 = "new_queue_1";
    let q2 = "new_queue_2";

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    // declare vhost 1
    run_succeeds(["declare", "vhost", "--name", vh1]);

    // declare vhost 2
    run_succeeds(["declare", "vhost", "--name", vh2]);

    // declare a new queue in vhost 1
    run_succeeds([
        "-V", vh1, "declare", "queue", "--name", q1, "--type", "classic",
    ]);

    // declare a new queue in vhost 2
    run_succeeds([
        "-V", vh2, "declare", "queue", "--name", q2, "--type", "quorum",
    ]);

    // bind the queue -> a pre-existing exchange
    run_succeeds([
        "-V",
        vh1,
        "declare",
        "binding",
        "--source",
        "amq.direct",
        "--destination_type",
        "queue",
        "--destination",
        q1,
        "--routing_key",
        "routing_key_queue",
    ]);

    // declare an exchange -> exchange binding
    run_succeeds([
        "-V",
        vh1,
        "declare",
        "binding",
        "--source",
        "amq.direct",
        "--destination_type",
        "exchange",
        "--destination",
        "amq.topic",
        "--routing_key",
        "routing_key_exchange",
    ]);

    await_queue_metric_emission();

    // list bindings in vhost 1
    run_succeeds(["-V", "bindings_vhost_1", "list", "bindings"]).stdout(
        predicate::str::contains("new_queue_1")
            .and(predicate::str::contains("routing_key_queue"))
            .and(predicate::str::contains("routing_key_exchange")),
    );

    // delete the queue from vhost 1
    run_succeeds(["-V", vh1, "delete", "queue", "--name", q1]);

    // these bindings were deleted with the queue
    run_succeeds(["-V", "bindings_vhost_1", "list", "bindings"]).stdout(
        predicate::str::contains("new_queue_1")
            .not()
            .and(predicate::str::contains("routing_key_queue"))
            .not()
            .and(predicate::str::contains("routing_key_exchange")),
    );

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    Ok(())
}
