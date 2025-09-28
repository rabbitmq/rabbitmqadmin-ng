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
fn list_streams() -> Result<(), Box<dyn std::error::Error>> {
    let vh1 = "rabbitmqadmin.stream_vhost_1";
    let vh2 = "rabbitmqadmin.stream_vhost_2";
    let s1 = "new_stream1";
    let s2 = "new_stream2";

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    // declare vhost 1
    run_succeeds(["declare", "vhost", "--name", vh1]);

    // declare vhost 2
    run_succeeds(["declare", "vhost", "--name", vh2]);

    // declare a new stream in vhost 1
    run_succeeds([
        "-V",
        vh1,
        "declare",
        "stream",
        "--name",
        s1,
        "--expiration",
        "2D",
    ]);

    // declare new stream in vhost 2
    run_succeeds([
        "-V",
        vh2,
        "declare",
        "stream",
        "--name",
        s2,
        "--expiration",
        "12h",
    ]);

    await_queue_metric_emission();

    // list streams in vhost 1
    run_succeeds(["-V", vh1, "list", "queues"])
        .stdout(output_includes(s1).and(output_includes("random_stream").not()));

    // delete the stream in vhost 1
    run_succeeds(["-V", vh1, "delete", "stream", "--name", s1]);

    // list streams in vhost 1
    run_succeeds(["-V", vh1, "list", "queues"]).stdout(output_includes(s1).not());

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn streams_list() -> Result<(), Box<dyn std::error::Error>> {
    let vh1 = "rabbitmqadmin.stream_vhost_3";
    let vh2 = "rabbitmqadmin.stream_vhost_4";
    let s1 = "new_stream1";
    let s2 = "new_stream2";

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    // declare vhost 1
    run_succeeds(["vhosts", "declare", "--name", vh1]);

    // declare vhost 2
    run_succeeds(["vhosts", "declare", "--name", vh2]);

    // declare a new stream in vhost 1
    run_succeeds([
        "-V",
        vh1,
        "streams",
        "declare",
        "--name",
        s1,
        "--expiration",
        "2D",
    ]);

    // declare new stream in vhost 2
    run_succeeds([
        "-V",
        vh2,
        "streams",
        "declare",
        "--name",
        s2,
        "--expiration",
        "12h",
    ]);

    await_queue_metric_emission();

    // list streams in vhost 1
    run_succeeds(["-V", vh1, "streams", "list"])
        .stdout(output_includes(s1).and(output_includes("random_stream").not()));

    // delete the stream in vhost 1
    run_succeeds(["-V", vh1, "streams", "delete", "--name", s1]);

    // list streams in vhost 1
    run_succeeds(["-V", vh1, "streams", "list"]).stdout(output_includes(s1).not());

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_streams_delete_idempotently() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "rabbitmqadmin.streams.test1";
    let s = "test_stream_delete_idempotently";

    delete_vhost(vh).expect("failed to delete a virtual host");
    run_succeeds(["declare", "vhost", "--name", vh]);

    run_succeeds(["-V", vh, "streams", "delete", "--name", s, "--idempotently"]);

    run_succeeds([
        "-V",
        vh,
        "declare",
        "stream",
        "--name",
        s,
        "--expiration",
        "2D",
    ]);

    run_succeeds(["-V", vh, "streams", "delete", "--name", s]);

    run_succeeds(["-V", vh, "streams", "delete", "--name", s, "--idempotently"]);

    run_succeeds([
        "declare",
        "stream",
        "-V",
        vh,
        "--name",
        s,
        "--expiration",
        "2D",
    ]);
    run_succeeds(["delete", "stream", "-V", vh, "--name", s]);
    run_succeeds(["delete", "stream", "-V", vh, "--name", s, "--idempotently"]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}
