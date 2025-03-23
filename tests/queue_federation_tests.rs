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
use rabbitmq_http_client::requests::{FederationUpstreamParams, QueueFederationParams};

mod test_helpers;
use crate::test_helpers::{amqp_endpoint_with_vhost, delete_vhost};
use test_helpers::{run_fails, run_succeeds};

#[test]
fn test_federation_upstream_declaration_for_queue_federation_case0()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "rust.federation.0";
    let name = "up.for_queue_federation";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.1";
    let ctag = "federation.custom-consumer-tag";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, &endpoint1, qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let qfp = upstream.queue_federation.unwrap();

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        &upstream.name,
        "--uri",
        &upstream.uri,
        "--queue-name",
        &q,
        "--consumer-tag",
        &qfp.consumer_tag.unwrap(),
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_queue_federation_case1a()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "rust.federation.1a";
    let name = "up.for_queue_federation.a";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.1a";
    let ctag = "federation.custom-consumer-tag.a";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, &endpoint1, qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let qfp = upstream.queue_federation.unwrap();

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        &upstream.name,
        "--uri",
        &upstream.uri,
        "--ack-mode",
        "on-confirm",
        "--queue-name",
        &q,
        "--consumer-tag",
        &qfp.consumer_tag.unwrap(),
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_queue_federation_case1b()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "rust.federation.1b";
    let name = "up.for_queue_federation.b";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.1b";
    let ctag = "federation.custom-consumer-tag.b";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, &endpoint1, qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let qfp = upstream.queue_federation.unwrap();

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream",
        "--name",
        &upstream.name,
        "--uri",
        &upstream.uri,
        "--ack-mode",
        "on-confirm",
        "--queue-name",
        &q,
        "--consumer-tag",
        &qfp.consumer_tag.unwrap(),
        // exchange federation
        "--queue-type",
        "quorum",
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_queue_federation_case2()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "rust.federation.2";
    let name = "up.for_queue_federation";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.2";
    let ctag = "federation.custom-consumer-tag.2";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, &endpoint1, qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        &upstream.name,
        "--uri",
        &upstream.uri,
        "--ack-mode",
        "on-publish",
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_queue_federation_case3()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "rust.federation.3";
    let name = "up.for_queue_federation";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.3";
    let ctag = "federation.custom-consumer-tag.3";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, &endpoint1, qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);

    // missing --uri
    run_fails([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        &upstream.name,
    ])
    .stderr(predicate::str::contains(
        "required arguments were not provided",
    ));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_queue_federation_case4()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "rust.federation.4";
    let name = "up.for_queue_federation";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.3";
    let ctag = "federation.custom-consumer-tag.3";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, &endpoint1, qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);

    // missing --name
    run_fails([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--uri",
        &upstream.uri,
        "--ack-mode",
        "on-publish",
    ])
    .stderr(predicate::str::contains(
        "required arguments were not provided",
    ));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_list_all_upstreams_with_queue_federation()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "rust.federation.5";
    let name = "up.for_queue_federation/5";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.5";
    let ctag = "federation.custom-consumer-tag";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, &endpoint1, qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let qfp = upstream.queue_federation.unwrap();

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        &upstream.name,
        "--uri",
        &upstream.uri,
        "--queue-name",
        &q,
        "--consumer-tag",
        &qfp.consumer_tag.unwrap(),
    ]);

    run_succeeds(["-V", vh, "federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(name))
        .stdout(predicate::str::contains(endpoint1.clone()))
        .stdout(predicate::str::contains(q))
        .stdout(predicate::str::contains(ctag));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_delete_an_upstream_with_queue_federation_settings()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "rust.federation.6";
    let name = "up.for_queue_federation.6";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.6";
    let ctag = "federation.custom-consumer-tag.6";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, &endpoint1, qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let qfp = upstream.queue_federation.unwrap();

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        &upstream.name,
        "--uri",
        &upstream.uri,
        "--queue-name",
        &q,
        "--consumer-tag",
        &qfp.consumer_tag.unwrap(),
    ]);

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(name))
        .stdout(predicate::str::contains(endpoint1.clone()));

    run_succeeds([
        "-V",
        vh,
        "federation",
        "delete_upstream",
        "--name",
        &upstream.name,
    ]);

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(name).not())
        .stdout(predicate::str::contains(endpoint1.clone()).not());

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}
