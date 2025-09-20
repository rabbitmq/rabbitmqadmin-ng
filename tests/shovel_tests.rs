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

mod test_helpers;

use crate::test_helpers::*;
use predicates::boolean::PredicateBooleanExt;
use predicates::prelude::predicate;

#[test]
#[ignore]
fn test_shovel_declaration_without_source_uri() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "rust.shovels.0";
    let name = "shovels.test_shovel_declaration_without_source_uri";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let src_q = "rust.shovels.src.q";
    let dest_x = "rust.shovels.dest.x";

    run_succeeds(["declare", "vhost", "--name", vh]);
    run_fails([
        "-V",
        vh,
        "shovels",
        "declare_amqp091",
        "--name",
        name,
        "--destination-uri",
        &amqp_endpoint,
        "--source-queue",
        src_q,
        "--destination-exchange",
        dest_x,
    ])
    .stderr(predicate::str::contains(
        "required arguments were not provided",
    ));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_shovel_declaration_without_destination_uri() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "rust.shovels.0";
    let name = "shovels.test_shovel_declaration_without_destination_uri";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let src_q = "rust.shovels.src.q";
    let dest_x = "rust.shovels.dest.x";

    run_succeeds(["declare", "vhost", "--name", vh]);
    run_fails([
        "-V",
        vh,
        "shovels",
        "declare_amqp091",
        "--name",
        name,
        "--source-uri",
        &amqp_endpoint,
        "--source-queue",
        src_q,
        "--destination-exchange",
        dest_x,
    ])
    .stderr(predicate::str::contains(
        "required arguments were not provided",
    ));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_shovel_declaration_with_overlapping_destination_types()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "rust.shovels.1";
    let name = "shovels.test_shovel_declaration_with_overlapping_destination_types";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let src_q = "rust.shovels.src.q";
    let dest_x = "rust.shovels.dest.x";

    run_succeeds(["declare", "vhost", "--name", vh]);
    run_fails([
        "-V",
        vh,
        "shovels",
        "declare_amqp091",
        "--name",
        name,
        "--source-uri",
        &amqp_endpoint,
        "--destination-uri",
        &amqp_endpoint,
        "--source-queue",
        src_q,
        "--source-exchange",
        src_q,
        "--destination-exchange",
        dest_x,
    ])
    .stderr(predicate::str::contains("cannot be used with"));

    run_succeeds(["-V", vh, "shovels", "delete", "--name", name]);

    run_succeeds(["-V", vh, "shovels", "delete", "--name", name]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_amqp091_shovel_declaration_and_deletion() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "rust.shovels.2";
    let name = "shovels.test_amqp091_shovel_declaration_and_deletion";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let src_q = "rust.shovels.src.q";
    let dest_x = "rust.shovels.dest.x";

    run_succeeds(["declare", "vhost", "--name", vh]);
    run_succeeds([
        "-V",
        vh,
        "shovels",
        "declare_amqp091",
        "--name",
        name,
        "--source-uri",
        &amqp_endpoint,
        "--destination-uri",
        &amqp_endpoint,
        "--source-queue",
        src_q,
        "--destination-exchange",
        dest_x,
    ]);

    run_succeeds(["-V", vh, "shovels", "list"]).stdout(
        predicate::str::contains(vh)
            .and(predicate::str::contains(src_q))
            .and(predicate::str::contains("dynamic"))
            .and(predicate::str::contains("node"))
            .and(predicate::str::contains("state")),
    );

    run_succeeds(["-V", vh, "shovels", "list_all"]).stdout(
        predicate::str::contains(vh)
            .and(predicate::str::contains(src_q))
            .and(predicate::str::contains("dynamic"))
            .and(predicate::str::contains("vhost"))
            .and(predicate::str::contains("node"))
            .and(predicate::str::contains("state")),
    );

    run_succeeds(["-V", vh, "shovels", "delete", "--name", name]);
    run_succeeds(["-V", vh, "shovels", "delete", "--name", name]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_amqp10_shovel_declaration_and_deletion() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "rust.shovels.3";
    let name = "shovels.test_amqp10_shovel_declaration_and_deletion";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let src_q = "rust.shovels.src.q";
    let dest_q = "rust.shovels.dest.q";

    run_succeeds(["declare", "vhost", "--name", vh]);
    run_succeeds([
        "--vhost", vh, "declare", "queue", "--name", src_q, "--type", "quorum",
    ]);
    run_succeeds([
        "--vhost", vh, "declare", "queue", "--name", dest_q, "--type", "quorum",
    ]);

    run_succeeds([
        "-V",
        vh,
        "shovels",
        "declare_amqp10",
        "--name",
        name,
        "--source-uri",
        &amqp_endpoint,
        "--destination-uri",
        &amqp_endpoint,
        "--source-address",
        format!("/queue/{}", src_q).as_str(),
        "--destination-address",
        format!("/queue/{}", dest_q).as_str(),
    ]);

    run_succeeds(["-V", vh, "shovels", "delete", "--name", name]);
    run_succeeds(["-V", vh, "shovels", "delete", "--name", name]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}
