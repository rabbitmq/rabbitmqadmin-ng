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
fn list_exchanges() -> Result<(), Box<dyn std::error::Error>> {
    let vh1 = "exchange_vhost_1";
    let vh2 = "exchange_vhost_2";

    let x1 = "new_exchange_1";
    let x2 = "new_exchange_2";

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    // declare vhost 1
    run_succeeds(["declare", "vhost", "--name", vh1]);

    // declare vhost 2
    run_succeeds(["declare", "vhost", "--name", vh2]);

    // declare a new exchange in vhost 1
    run_succeeds(["-V", vh1, "declare", "exchange", "--name", x1]);

    // declare a new exchange in vhost 2
    run_succeeds(["-V", vh2, "declare", "exchange", "--name", x2]);

    // list exchanges in vhost 1
    run_succeeds(["-V", vh1, "list", "exchanges"]).stdout(
        output_includes("amq.direct")
            .and(output_includes("amq.fanout"))
            .and(output_includes(x1))
            .and(output_includes(x2).not()),
    );

    // delete the exchanges from vhost 1
    run_succeeds(["-V", vh1, "delete", "exchange", "--name", x1]);

    // list exchange in vhost 1
    run_succeeds(["-V", vh1, "list", "exchanges"]).stdout(
        output_includes("amq.direct")
            .and(output_includes("amq.topic"))
            .and(output_includes(x1).not()),
    );

    // list exchange in vhost 2
    run_succeeds(["-V", vh2, "list", "exchanges"]).stdout(
        output_includes("amq.direct")
            .and(output_includes("amq.headers"))
            .and(output_includes(x2))
            .and(output_includes(x1).not()),
    );

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn exchanges_list() -> Result<(), Box<dyn std::error::Error>> {
    let vh1 = "exchange_vhost_3";
    let vh2 = "exchange_vhost_4";

    let x1 = "new_exchange_1";
    let x2 = "new_exchange_2";

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    // declare vhost 1
    run_succeeds(["vhosts", "declare", "--name", vh1]);

    // declare vhost 2
    run_succeeds(["vhosts", "declare", "--name", vh2]);

    // declare a new exchange in vhost 1
    run_succeeds(["-V", vh1, "exchanges", "declare", "--name", x1]);

    // declare a new exchange in vhost 2
    run_succeeds(["-V", vh2, "exchanges", "declare", "--name", x2]);

    // list exchanges in vhost 1
    run_succeeds(["-V", vh1, "exchanges", "list"]).stdout(
        output_includes("amq.direct")
            .and(output_includes("amq.fanout"))
            .and(output_includes(x1))
            .and(output_includes(x2).not()),
    );

    // delete the exchanges from vhost 1
    run_succeeds(["-V", vh1, "exchanges", "delete", "--name", x1]);

    // list exchange in vhost 1
    run_succeeds(["-V", vh1, "exchanges", "list"]).stdout(
        output_includes("amq.direct")
            .and(output_includes("amq.topic"))
            .and(output_includes(x1).not()),
    );

    // list exchange in vhost 2
    run_succeeds(["-V", vh2, "exchanges", "list"]).stdout(
        output_includes("amq.direct")
            .and(output_includes("amq.headers"))
            .and(output_includes(x2))
            .and(output_includes(x1).not()),
    );

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn delete_an_existing_exchange_using_original_command_group()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "rabbitmqadmin.exchanges.test1";
    let x = "exchange_1_to_delete";

    // create a vhost
    create_vhost(vh)?;

    // declare an exchange
    run_succeeds(["-V", vh, "declare", "exchange", "--name", x]);

    // list exchanges in vhost 1
    run_succeeds(["-V", vh, "list", "exchanges"]).stdout(output_includes(x));

    // delete the exchange
    run_succeeds(["-V", vh, "delete", "exchange", "--name", x]);

    // list exchange in vhost 1
    run_succeeds(["-V", vh, "list", "exchanges"]).stdout(output_includes(x).not());

    // delete the vhost
    delete_vhost(vh)?;

    Ok(())
}

#[test]
fn delete_an_existing_exchange_using_exchanges_command_group()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "rabbitmqadmin.exchanges.test2";
    let x = "exchange_1_to_delete";

    // create a vhost
    create_vhost(vh)?;

    // declare an exchange
    run_succeeds(["-V", vh, "exchanges", "declare", "--name", x]);

    // list exchanges in vhost 1
    run_succeeds(["-V", vh, "exchanges", "list"]).stdout(output_includes(x));

    // delete the exchange
    run_succeeds(["-V", vh, "exchanges", "delete", "--name", x]);

    // list exchange in vhost 1
    run_succeeds(["-V", vh, "exchanges", "list"]).stdout(output_includes(x).not());

    // delete the vhost
    delete_vhost(vh)?;

    Ok(())
}

#[test]
fn delete_a_non_existing_exchange() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "rabbitmqadmin.exchanges.test3";

    // declare a vhost
    create_vhost(vh)?;

    // try deleting a non-existent exchange with --idempotently
    run_succeeds([
        "--vhost",
        vh,
        "exchanges",
        "delete",
        "--name",
        "7s98df7s79df-non-existent",
        "--idempotently",
    ]);

    // try deleting it without
    run_fails([
        "--vhost",
        vh,
        "exchanges",
        "delete",
        "--name",
        "7s98df7s79df-non-existent",
    ])
    .stderr(output_includes("Not Found"));

    // delete the vhost
    delete_vhost(vh)?;

    Ok(())
}

#[test]
fn test_exchanges_bind_and_unbind() -> Result<(), Box<dyn std::error::Error>> {
    let vh1 = "exchanges_bind_vhost_3";
    let vh2 = "exchanges_bind_vhost_4";
    let q1 = "new_queue_1";
    let q2 = "new_queue_2";

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    // declare vhost 1
    run_succeeds(["vhosts", "declare", "--name", vh1]);

    // declare vhost 2
    run_succeeds(["vhosts", "declare", "--name", vh2]);

    // declare a new queue in vhost 1
    run_succeeds([
        "-V", vh1, "queues", "declare", "--name", q1, "--type", "classic",
    ]);

    // declare a new queue in vhost 2
    run_succeeds([
        "-V", vh2, "queues", "declare", "--name", q2, "--type", "quorum",
    ]);

    // bind the queue -> a pre-existing exchange
    run_succeeds([
        "-V",
        vh1,
        "exchanges",
        "bind",
        "--source",
        "amq.direct",
        "--destination-type",
        "queue",
        "--destination",
        q1,
        "--routing-key",
        "routing_key_queue",
    ]);

    // declare an exchange -> exchange binding
    run_succeeds([
        "-V",
        vh1,
        "exchanges",
        "bind",
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

    // list bindings in vhost 1
    run_succeeds(["-V", vh2, "list", "bindings"]).stdout(
        output_includes("new_queue_1")
            .and(output_includes("routing_key_queue"))
            .and(output_includes("routing_key_exchange")),
    );

    // unbind
    run_succeeds([
        "-V",
        vh1,
        "exchanges",
        "unbind",
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
