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
fn test_list_bindings() -> Result<(), Box<dyn Error>> {
    let vh1 = "rabbitmqadmin.test_list_bindings_1";
    let vh2 = "rabbitmqadmin.test_list_bindings_2";
    let q1 = "new_queue_1";
    let q2 = "new_queue_2";

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

    run_succeeds([
        "-V",
        vh1,
        "declare",
        "binding",
        "--source",
        "amq.direct",
        "--destination-type",
        "queue",
        "--destination",
        q1,
        "--routing-key",
        "routing_key_queue",
    ]);

    run_succeeds([
        "-V",
        vh1,
        "declare",
        "binding",
        "--source",
        "amq.direct",
        "--destination-type",
        "exchange",
        "--destination",
        "amq.topic",
        "--routing-key",
        "routing_key_exchange",
    ]);

    await_queue_metric_emission();

    run_succeeds(["-V", "bindings_vhost_1", "list", "bindings"]).stdout(
        output_includes("new_queue_1")
            .and(output_includes("routing_key_queue"))
            .and(output_includes("routing_key_exchange")),
    );

    run_succeeds(["-V", vh1, "queues", "delete", "--name", q1]);

    run_succeeds(["-V", "bindings_vhost_1", "list", "bindings"]).stdout(
        output_includes("new_queue_1")
            .not()
            .and(output_includes("routing_key_queue"))
            .not()
            .and(output_includes("routing_key_exchange")),
    );

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_bindings_list() -> Result<(), Box<dyn Error>> {
    let vh1 = "rabbitmqadmin.test_bindings_list_1";
    let vh2 = "rabbitmqadmin.test_bindings_list_2";
    let q1 = "new_queue_1";
    let q2 = "new_queue_2";

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

    run_succeeds([
        "-V",
        vh1,
        "bindings",
        "declare",
        "--source",
        "amq.direct",
        "--destination-type",
        "queue",
        "--destination",
        q1,
        "--routing-key",
        "routing_key_queue",
    ]);

    run_succeeds([
        "-V",
        vh1,
        "bindings",
        "declare",
        "--source",
        "amq.direct",
        "--destination-type",
        "exchange",
        "--destination",
        "amq.topic",
        "--routing-key",
        "routing_key_exchange",
    ]);

    await_queue_metric_emission();

    run_succeeds(["-V", vh1, "list", "bindings"]).stdout(
        output_includes("new_queue_1")
            .and(output_includes("routing_key_queue"))
            .and(output_includes("routing_key_exchange")),
    );

    run_succeeds([
        "-V",
        vh1,
        "bindings",
        "declare",
        "--source",
        "amq.direct",
        "--destination-type",
        "queue",
        "--destination",
        q1,
        "--routing-key",
        "routing_key_queue",
    ]);

    run_succeeds(["-V", vh1, "list", "bindings"]).stdout(
        output_includes("new_queue_1")
            .not()
            .and(output_includes("routing_key_queue"))
            .not()
            .and(output_includes("routing_key_exchange")),
    );

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_bindings_delete_idempotently() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.bindings.test1";
    let source_ex = "test_source_exchange";
    let dest_queue = "test_dest_queue";
    let routing_key = "test.routing.key";

    delete_vhost(vh).expect("failed to delete a virtual host");
    run_succeeds(["declare", "vhost", "--name", vh]);

    run_succeeds([
        "-V",
        vh,
        "bindings",
        "delete",
        "--source",
        source_ex,
        "--destination-type",
        "queue",
        "--destination",
        dest_queue,
        "--routing-key",
        routing_key,
        "--idempotently",
    ]);

    run_succeeds([
        "-V", vh, "declare", "exchange", "--name", source_ex, "--type", "direct",
    ]);
    run_succeeds([
        "-V", vh, "declare", "queue", "--name", dest_queue, "--type", "classic",
    ]);

    run_succeeds([
        "-V",
        vh,
        "bindings",
        "declare",
        "--source",
        source_ex,
        "--destination-type",
        "queue",
        "--destination",
        dest_queue,
        "--routing-key",
        routing_key,
    ]);

    run_succeeds([
        "-V",
        vh,
        "bindings",
        "delete",
        "--source",
        source_ex,
        "--destination-type",
        "queue",
        "--destination",
        dest_queue,
        "--routing-key",
        routing_key,
    ]);

    run_succeeds([
        "-V",
        vh,
        "bindings",
        "delete",
        "--source",
        source_ex,
        "--destination-type",
        "queue",
        "--destination",
        dest_queue,
        "--routing-key",
        routing_key,
        "--idempotently",
    ]);

    run_succeeds([
        "-V",
        vh,
        "bindings",
        "declare",
        "--source",
        source_ex,
        "--destination-type",
        "queue",
        "--destination",
        dest_queue,
        "--routing-key",
        routing_key,
    ]);
    run_succeeds([
        "-V",
        vh,
        "delete",
        "binding",
        "--source",
        source_ex,
        "--destination-type",
        "queue",
        "--destination",
        dest_queue,
        "--routing-key",
        routing_key,
    ]);
    run_succeeds([
        "-V",
        vh,
        "delete",
        "binding",
        "--source",
        source_ex,
        "--destination-type",
        "queue",
        "--destination",
        dest_queue,
        "--routing-key",
        routing_key,
        "--idempotently",
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}
