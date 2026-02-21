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
use rabbitmq_http_client::requests::{FederationUpstreamParams, QueueFederationParams};
use std::error::Error;

use crate::test_helpers::{
    amqp_endpoint_with_vhost, await_ms, delete_vhost, output_includes, rabbitmq_version_is_at_least,
};
use crate::test_helpers::{run_fails, run_succeeds};

#[test]
fn test_federation_upstream_declaration_for_queue_federation_case0() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.queue.test1";
    let name = "up.for_queue_federation";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.1";
    let ctag = "federation.custom-consumer-tag";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, endpoint1.as_str(), qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let qfp = upstream.queue_federation.unwrap();

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        upstream.name,
        "--uri",
        upstream.uri,
        "--queue-name",
        q,
        "--consumer-tag",
        qfp.consumer_tag.unwrap(),
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_queue_federation_case1a() -> Result<(), Box<dyn Error>>
{
    let vh = "rabbitmqadmin.federation.queue.test2";
    let name = "up.for_queue_federation.a";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.1a";
    let ctag = "federation.custom-consumer-tag.a";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, endpoint1.as_str(), qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let qfp = upstream.queue_federation.unwrap();

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        upstream.name,
        "--uri",
        upstream.uri,
        "--ack-mode",
        "on-confirm",
        "--queue-name",
        q,
        "--consumer-tag",
        qfp.consumer_tag.unwrap(),
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_queue_federation_case1b() -> Result<(), Box<dyn Error>>
{
    let vh = "rabbitmqadmin.federation.queue.test3";
    let name = "up.for_queue_federation.b";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.1b";
    let ctag = "federation.custom-consumer-tag.b";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, endpoint1.as_str(), qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let qfp = upstream.queue_federation.unwrap();
    let consumer_tag = qfp.consumer_tag.unwrap();

    let mut args = vec![
        "-V",
        vh,
        "federation",
        "declare_upstream",
        "--name",
        upstream.name,
        "--uri",
        upstream.uri,
        "--ack-mode",
        "on-confirm",
        "--queue-name",
        q,
        "--consumer-tag",
        consumer_tag,
    ];
    // --queue-type requires RabbitMQ 3.13+
    if rabbitmq_version_is_at_least(3, 13, 0) {
        args.extend(["--queue-type", "quorum"]);
    }
    run_succeeds(args);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_queue_federation_case2() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.queue.test4";
    let name = "up.for_queue_federation";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.2";
    let ctag = "federation.custom-consumer-tag.2";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, endpoint1.as_str(), qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        upstream.name,
        "--uri",
        upstream.uri,
        "--ack-mode",
        "on-publish",
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_queue_federation_case3() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.queue.test5";
    let name = "up.for_queue_federation";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.3";
    let ctag = "federation.custom-consumer-tag.3";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, endpoint1.as_str(), qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);

    // missing --uri
    run_fails([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        upstream.name,
    ])
    .stderr(output_includes("required arguments were not provided"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_queue_federation_case4() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.queue.test6";
    let name = "up.for_queue_federation";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.3";
    let ctag = "federation.custom-consumer-tag.3";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, endpoint1.as_str(), qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);

    // missing --name
    run_fails([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--uri",
        upstream.uri,
        "--ack-mode",
        "on-publish",
    ])
    .stderr(output_includes("required arguments were not provided"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_list_all_upstreams_with_queue_federation() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.queue.test7";
    let name = "up.for_queue_federation/5";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.5";
    let ctag = "federation.custom-consumer-tag";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, endpoint1.as_str(), qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let qfp = upstream.queue_federation.unwrap();

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        upstream.name,
        "--uri",
        upstream.uri,
        "--queue-name",
        q,
        "--consumer-tag",
        qfp.consumer_tag.unwrap(),
    ]);

    run_succeeds(["-V", vh, "federation", "list_all_upstreams"])
        .stdout(output_includes(name))
        .stdout(output_includes(endpoint1.as_str()))
        .stdout(output_includes(q))
        .stdout(output_includes(ctag));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_delete_an_upstream_with_queue_federation_settings() -> Result<(), Box<dyn Error>>
{
    let vh = "rabbitmqadmin.federation.queue.test8";
    let name = "up.for_queue_federation.6";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let q = "federation.cq.6";
    let ctag = "federation.custom-consumer-tag.6";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, endpoint1.as_str(), qfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let qfp = upstream.queue_federation.unwrap();

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        upstream.name,
        "--uri",
        upstream.uri,
        "--queue-name",
        q,
        "--consumer-tag",
        qfp.consumer_tag.unwrap(),
    ]);

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(output_includes(name))
        .stdout(output_includes(endpoint1.as_str()));

    run_succeeds([
        "-V",
        vh,
        "federation",
        "delete_upstream",
        "--name",
        upstream.name,
    ]);

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(output_includes(name).not())
        .stdout(output_includes(endpoint1.as_str()).not());

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_list_all_links_with_queue_federation_settings() -> Result<(), Box<dyn Error>> {
    let vh1 = "rabbitmqadmin.federation.links.a";
    let vh2 = "rabbitmqadmin.federation.links.b";
    let name = "up.for_queue_federation.links.a";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh2);
    let q = "federation.cq.a";
    let ctag = "federation.custom-consumer-tag.b";
    let qfp = QueueFederationParams::new_with_consumer_tag(q, ctag);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_queue_federation_upstream(vh1, name, endpoint1.as_str(), qfp);

    run_succeeds(["declare", "vhost", "--name", vh1]);
    run_succeeds(["declare", "vhost", "--name", vh2]);
    let qfp = upstream.queue_federation.unwrap();

    run_succeeds([
        "-V",
        vh1,
        "federation",
        "declare_upstream",
        "--name",
        upstream.name,
        "--uri",
        upstream.uri,
        "--ack-mode",
        "on-confirm",
        "--queue-name",
        q,
        "--consumer-tag",
        qfp.consumer_tag.unwrap(),
    ]);

    run_succeeds([
        "-V", vh1, "declare", "queue", "--name", q, "--type", "classic",
    ]);

    run_succeeds([
        "-V",
        vh1,
        "policies",
        "declare",
        "--name",
        name,
        "--pattern",
        "^federation\\.",
        "--applies-to",
        "queues",
        "--priority",
        "98",
        "--definition",
        "{\"federation-upstream-set\": \"all\"}",
    ]);

    await_ms(1000);

    run_succeeds(["federation", "list_all_links"])
        .stdout(output_includes(name))
        .stdout(output_includes(vh1))
        .stdout(output_includes(ctag));

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    Ok(())
}
