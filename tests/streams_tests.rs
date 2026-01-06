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

mod test_helpers;
use crate::test_helpers::*;

#[test]
fn list_streams() -> Result<(), Box<dyn Error>> {
    let vh1 = "rabbitmqadmin.stream_vhost_1";
    let vh2 = "rabbitmqadmin.stream_vhost_2";
    let s1 = "new_stream1";
    let s2 = "new_stream2";

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    run_succeeds(["declare", "vhost", "--name", vh1]);

    run_succeeds(["declare", "vhost", "--name", vh2]);

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

    run_succeeds(["-V", vh1, "list", "queues"])
        .stdout(output_includes(s1).and(output_includes("random_stream").not()));

    run_succeeds(["-V", vh1, "delete", "stream", "--name", s1]);

    run_succeeds(["-V", vh1, "list", "queues"]).stdout(output_includes(s1).not());

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn streams_list() -> Result<(), Box<dyn Error>> {
    let vh1 = "rabbitmqadmin.stream_vhost_3";
    let vh2 = "rabbitmqadmin.stream_vhost_4";
    let s1 = "new_stream1";
    let s2 = "new_stream2";

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    run_succeeds(["vhosts", "declare", "--name", vh1]);

    run_succeeds(["vhosts", "declare", "--name", vh2]);

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

    run_succeeds(["-V", vh1, "streams", "list"])
        .stdout(output_includes(s1).and(output_includes("random_stream").not()));

    run_succeeds(["-V", vh1, "streams", "delete", "--name", s1]);

    run_succeeds(["-V", vh1, "streams", "list"]).stdout(output_includes(s1).not());

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_streams_delete_idempotently() -> Result<(), Box<dyn Error>> {
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

#[test]
fn test_streams_list_with_columns() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.streams.columns_test";
    let s = "test_stream_columns";

    delete_vhost(vh).expect("failed to delete a virtual host");
    run_succeeds(["declare", "vhost", "--name", vh]);

    run_succeeds([
        "-V",
        vh,
        "streams",
        "declare",
        "--name",
        s,
        "--expiration",
        "2D",
    ]);

    await_queue_metric_emission();

    run_succeeds(["-V", vh, "streams", "list", "--columns", "name,queue_type"])
        .stdout(output_includes(s).and(output_includes("stream")));

    run_succeeds(["-V", vh, "streams", "list", "--columns", "name"]).stdout(output_includes(s));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_streams_show() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.streams.show_test";
    let s = "test_stream_show";

    let _ = delete_vhost(vh);
    run_succeeds(["declare", "vhost", "--name", vh]);

    run_succeeds([
        "-V",
        vh,
        "streams",
        "declare",
        "--name",
        s,
        "--expiration",
        "2D",
    ]);

    await_queue_metric_emission();

    run_succeeds(["-V", vh, "streams", "show", "--name", s])
        .stdout(output_includes(s).and(output_includes("stream")));

    run_succeeds([
        "-V",
        vh,
        "streams",
        "show",
        "--name",
        s,
        "--columns",
        "name,queue_type",
    ])
    .stdout(output_includes(s).and(output_includes("stream")));

    let _ = delete_vhost(vh);

    Ok(())
}
