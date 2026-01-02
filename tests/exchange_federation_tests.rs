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
use rabbitmq_http_client::commons::QueueType;
use rabbitmq_http_client::requests::{ExchangeFederationParams, FederationUpstreamParams};
use std::error::Error;

mod test_helpers;
use crate::test_helpers::{amqp_endpoint_with_vhost, delete_vhost, output_includes};
use test_helpers::{run_fails, run_succeeds};

#[test]
fn test_federation_upstream_declaration_for_exchange_federation_case0() -> Result<(), Box<dyn Error>>
{
    let vh = "rabbitmqadmin.federation.exchange.test1";
    let name = "up.for_exchange_federation.0";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let queue_type = QueueType::Quorum;
    let xfp = ExchangeFederationParams::new(queue_type);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_exchange_federation_upstream(vh, name, &endpoint1, xfp);

    run_succeeds(["declare", "vhost", "--name", vh]);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        upstream.name,
        "--uri",
        upstream.uri,
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_exchange_federation_case1a()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.exchange.test2";
    let name = "up.for_exchange_federation.1a";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let x = "federation.x.1a";
    let queue_type = QueueType::Quorum;
    let xfp = ExchangeFederationParams::new(queue_type);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_exchange_federation_upstream(vh, name, &endpoint1, xfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let xfp = upstream.exchange_federation.unwrap();

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        upstream.name,
        "--uri",
        upstream.uri,
        "--exchange-name",
        x,
        "--queue-type",
        &xfp.queue_type.to_string(),
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_exchange_federation_case1b()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.exchange.test3";
    let name = "up.for_exchange_federation.1b";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let x = "federation.x.1b";
    let queue_type = QueueType::Quorum;
    let xfp = ExchangeFederationParams::new(queue_type);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_exchange_federation_upstream(vh, name, &endpoint1, xfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let xfp = upstream.exchange_federation.unwrap();

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream",
        "--name",
        upstream.name,
        "--uri",
        upstream.uri,
        "--exchange-name",
        x,
        "--queue-type",
        &xfp.queue_type.to_string(),
        // queue federation
        "--queue-name",
        "overridden.queue.name",
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_exchange_federation_case2() -> Result<(), Box<dyn Error>>
{
    let vh = "rabbitmqadmin.federation.exchange.test4";
    let name = "up.for_exchange_federation.2";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let x = "federation.x.2";
    let queue_type = QueueType::Quorum;
    let xfp = ExchangeFederationParams::new(queue_type);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_exchange_federation_upstream(vh, name, &endpoint1, xfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let xfp = upstream.exchange_federation.unwrap();

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        upstream.name,
        "--uri",
        upstream.uri,
        "--exchange-name",
        x,
        "--queue-type",
        &xfp.queue_type.to_string(),
        "--max-hops",
        "2",
        "--ttl",
        "900000000",
        "--message-ttl",
        "450000000",
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_exchange_federation_case3() -> Result<(), Box<dyn Error>>
{
    let vh = "rabbitmqadmin.federation.exchange.test5";
    let name = "up.for_exchange_federation.3";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let x = "federation.x.3";
    let queue_type = QueueType::Quorum;
    let xfp = ExchangeFederationParams::new(queue_type);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_exchange_federation_upstream(vh, name, &endpoint1, xfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let xfp = upstream.exchange_federation.unwrap();

    // missing --name
    run_fails([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--uri",
        upstream.uri,
        "--exchange-name",
        x,
        "--queue-type",
        &xfp.queue_type.to_string(),
    ])
    .stderr(output_includes("required arguments were not provided"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_upstream_declaration_for_exchange_federation_case4() -> Result<(), Box<dyn Error>>
{
    let vh = "rabbitmqadmin.federation.exchange.test6";
    let name = "up.for_exchange_federation.4";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let x = "federation.x.4";
    let queue_type = QueueType::Quorum;
    let xfp = ExchangeFederationParams::new(queue_type);
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_exchange_federation_upstream(vh, name, &endpoint1, xfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let xfp = upstream.exchange_federation.unwrap();

    // missing --uri
    run_fails([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        upstream.name,
        "--exchange-name",
        x,
        "--queue-type",
        &xfp.queue_type.to_string(),
        "--max-hops",
        "2",
        "--ttl",
        "900000000",
        "--message-ttl",
        "450000000",
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_list_all_upstreams_with_exchange_federation() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.exchange.test7";
    let name = "up.for_exchange_federation.5";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let x = "federation.x.5";
    let queue_type = QueueType::Classic;
    let xfp = ExchangeFederationParams::new(queue_type.clone());
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_exchange_federation_upstream(vh, name, &endpoint1, xfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let xfp = upstream.exchange_federation.unwrap();

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        upstream.name,
        "--uri",
        upstream.uri,
        "--exchange-name",
        x,
        "--queue-type",
        &xfp.queue_type.to_string(),
        "--max-hops",
        "2",
        "--ttl",
        "900000000",
        "--message-ttl",
        "450000000",
    ]);

    run_succeeds(["-V", vh, "federation", "list_all_upstreams"])
        .stdout(output_includes(name))
        .stdout(output_includes(&endpoint1))
        .stdout(output_includes(x))
        .stdout(output_includes(&queue_type.to_string()));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_delete_an_upstream_with_exchange_federation_settings()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.exchange.test8";
    let name = "up.for_exchange_federation.6";

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let x = "federation.x.6";
    let queue_type = QueueType::Classic;
    let xfp = ExchangeFederationParams::new(queue_type.clone());
    let endpoint1 = amqp_endpoint.clone();
    let upstream =
        FederationUpstreamParams::new_exchange_federation_upstream(vh, name, &endpoint1, xfp);

    run_succeeds(["declare", "vhost", "--name", vh]);
    let xfp = upstream.exchange_federation.unwrap();

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        upstream.name,
        "--uri",
        upstream.uri,
        "--exchange-name",
        x,
        "--queue-type",
        &xfp.queue_type.to_string(),
        "--max-hops",
        "2",
        "--ttl",
        "900000000",
        "--message-ttl",
        "450000000",
    ]);

    run_succeeds(["-V", vh, "federation", "list_all_upstreams"])
        .stdout(output_includes(name))
        .stdout(output_includes(&endpoint1))
        .stdout(output_includes(x))
        .stdout(output_includes(&queue_type.to_string()));

    run_succeeds([
        "-V",
        vh,
        "federation",
        "delete_upstream",
        "--name",
        upstream.name,
    ]);

    run_succeeds(["-V", vh, "federation", "list_all_upstreams"])
        .stdout(output_includes(name).not());

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_federation_delete_upstream_idempotently() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.exchange.test9";
    let upstream_name = "test_upstream_delete_idempotently";

    delete_vhost(vh).expect("failed to delete a virtual host");
    run_succeeds(["declare", "vhost", "--name", vh]);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "delete_upstream",
        "--name",
        upstream_name,
        "--idempotently",
    ]);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream",
        "--name",
        upstream_name,
        "--uri",
        "amqp://guest@localhost",
    ]);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "delete_upstream",
        "--name",
        upstream_name,
    ]);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "delete_upstream",
        "--name",
        upstream_name,
        "--idempotently",
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}
