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

use crate::test_helpers::{
    api_client, await_metric_emission, delete_vhost, rabbitmq_version_is_at_least, run_succeeds,
};
use std::error::Error;
#[test]
fn test_disable_tls_peer_verification_for_all_upstreams_basic() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.modifications.test1";
    let upstream_name = "test_basic_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_endpoint = format!("amqp://localhost:5672/{}", vh);

    let mut args = vec![
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        upstream_name,
        "--uri",
        &amqp_endpoint,
        "--exchange-name",
        "x.fanout",
    ];
    if rabbitmq_version_is_at_least(3, 13, 0) {
        args.extend(["--queue-type", "classic"]);
    }
    run_succeeds(args);

    run_succeeds([
        "federation",
        "disable_tls_peer_verification_for_all_upstreams",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let upstream_param = params
        .iter()
        .find(|p| p.name == upstream_name && p.is_federation_upstream())
        .expect("Federation upstream parameter should exist");

    let uri = upstream_param.value["uri"]
        .as_str()
        .expect("uri should be a string");
    assert!(uri.contains("verify=verify_none"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_upstreams_with_existing_verify_param()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.modifications.test2";
    let upstream_name = "test_existing_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_endpoint = format!("amqp://localhost:5672/{}", vh);
    let source_uri = format!(
        "{}?key1=abc&verify=verify_peer&cacertfile=/path/to/ca_bundle.pem&key2=def&certfile=/path/to/client.pem&keyfile=/path/to/client.key&server_name_indication=example.com&custom_param=value123&another_param=xyz&heartbeat=60",
        amqp_endpoint
    );

    let mut args = vec![
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        upstream_name,
        "--uri",
        &source_uri,
        "--exchange-name",
        "x.fanout",
    ];
    if rabbitmq_version_is_at_least(3, 13, 0) {
        args.extend(["--queue-type", "classic"]);
    }
    run_succeeds(args);
    await_metric_emission(500);

    run_succeeds([
        "federation",
        "disable_tls_peer_verification_for_all_upstreams",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let upstream_param = params
        .iter()
        .find(|p| p.name == upstream_name && p.is_federation_upstream())
        .expect("Federation upstream parameter should exist");

    let uri = upstream_param.value["uri"]
        .as_str()
        .expect("uri should be a string");

    assert!(uri.contains("verify=verify_none"));
    assert!(!uri.contains("verify=verify_peer"));
    assert!(uri.contains("key1=abc"));
    assert!(uri.contains("key2=def"));
    assert!(uri.contains("cacertfile=/path/to/ca_bundle.pem"));
    assert!(uri.contains("certfile=/path/to/client.pem"));
    assert!(uri.contains("keyfile=/path/to/client.key"));
    assert!(uri.contains("server_name_indication=example.com"));
    assert!(uri.contains("custom_param=value123"));
    assert!(uri.contains("another_param=xyz"));
    assert!(uri.contains("heartbeat=60"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_upstreams_queue_federation_basic()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.modifications.test3";
    let upstream_name = "test_queue_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_endpoint = format!("amqp://localhost:5672/{}", vh);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        upstream_name,
        "--uri",
        &amqp_endpoint,
        "--queue-name",
        "test.queue",
        "--consumer-tag",
        "test-consumer",
    ]);

    run_succeeds([
        "federation",
        "disable_tls_peer_verification_for_all_upstreams",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let upstream_param = params
        .iter()
        .find(|p| p.name == upstream_name && p.is_federation_upstream())
        .expect("Federation upstream parameter should exist");

    let uri = upstream_param.value["uri"]
        .as_str()
        .expect("uri should be a string");
    assert!(uri.contains("verify=verify_none"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_upstreams_queue_federation_with_params()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.modifications.test4";
    let upstream_name = "test_queue_upstream_with_params";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_endpoint = format!("amqp://localhost:5672/{}", vh);
    let source_uri = format!(
        "{}?queue_param=test123&verify=verify_peer&cacertfile=/etc/ssl/certs/ca.pem&consumer_tag_param=custom&prefetch=100&ack_mode=on-confirm",
        amqp_endpoint
    );

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        upstream_name,
        "--uri",
        &source_uri,
        "--queue-name",
        "federated.queue",
        "--consumer-tag",
        "queue-consumer",
    ]);
    await_metric_emission(500);

    run_succeeds([
        "federation",
        "disable_tls_peer_verification_for_all_upstreams",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let upstream_param = params
        .iter()
        .find(|p| p.name == upstream_name && p.is_federation_upstream())
        .expect("Federation upstream parameter should exist");

    let uri = upstream_param.value["uri"]
        .as_str()
        .expect("uri should be a string");

    assert!(uri.contains("verify=verify_none"));
    assert!(uri.contains("queue_param=test123"));
    assert!(uri.contains("cacertfile=/etc/ssl/certs/ca.pem"));
    assert!(uri.contains("consumer_tag_param=custom"));
    assert!(uri.contains("prefetch=100"));
    assert!(uri.contains("ack_mode=on-confirm"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_upstreams_mixed_federation()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.modifications.test5";
    let exchange_upstream_name = "exchange_upstream";
    let queue_upstream_name = "queue_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_endpoint = format!("amqp://localhost:5672/{}", vh);
    let exchange_uri = format!(
        "{}?exchange_param=value1&verify=verify_peer&certfile=/path/to/client.pem",
        amqp_endpoint
    );
    let queue_uri = format!(
        "{}?queue_param=value2&verify=verify_peer&keyfile=/path/to/client.key",
        amqp_endpoint
    );

    let mut args = vec![
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        exchange_upstream_name,
        "--uri",
        &exchange_uri,
        "--exchange-name",
        "x.federated",
    ];
    if rabbitmq_version_is_at_least(3, 13, 0) {
        args.extend(["--queue-type", "classic"]);
    }
    run_succeeds(args);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        queue_upstream_name,
        "--uri",
        &queue_uri,
        "--queue-name",
        "q.federated",
        "--consumer-tag",
        "mixed-consumer",
    ]);
    await_metric_emission(500);

    run_succeeds([
        "federation",
        "disable_tls_peer_verification_for_all_upstreams",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;

    let exchange_upstream_param = params
        .iter()
        .find(|p| p.name == exchange_upstream_name && p.is_federation_upstream())
        .expect("Exchange upstream parameter should exist");
    let queue_upstream_param = params
        .iter()
        .find(|p| p.name == queue_upstream_name && p.is_federation_upstream())
        .expect("Queue upstream parameter should exist");

    let exchange_uri = exchange_upstream_param.value["uri"]
        .as_str()
        .expect("uri should be a string");
    let queue_uri = queue_upstream_param.value["uri"]
        .as_str()
        .expect("uri should be a string");

    assert!(exchange_uri.contains("verify=verify_none"));
    assert!(exchange_uri.contains("exchange_param=value1"));
    assert!(exchange_uri.contains("certfile=/path/to/client.pem"));

    assert!(queue_uri.contains("verify=verify_none"));
    assert!(queue_uri.contains("queue_param=value2"));
    assert!(queue_uri.contains("keyfile=/path/to/client.key"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_enable_tls_peer_verification_for_all_upstreams_basic() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.modifications.test6";
    let upstream_name = "test_enable_basic_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_endpoint = format!("amqp://localhost:5672/{}", vh);

    let mut args = vec![
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        upstream_name,
        "--uri",
        &amqp_endpoint,
        "--exchange-name",
        "x.fanout",
    ];
    if rabbitmq_version_is_at_least(3, 13, 0) {
        args.extend(["--queue-type", "classic"]);
    }
    run_succeeds(args);

    run_succeeds([
        "federation",
        "enable_tls_peer_verification_for_all_upstreams",
        "--node-local-ca-certificate-bundle-path",
        "/etc/ssl/certs/ca_bundle.pem",
        "--node-local-client-certificate-file-path",
        "/etc/ssl/certs/client.pem",
        "--node-local-client-private-key-file-path",
        "/etc/ssl/private/client.key",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let upstream_param = params
        .iter()
        .find(|p| p.name == upstream_name && p.is_federation_upstream())
        .expect("Federation upstream parameter should exist");

    let uri = upstream_param.value["uri"]
        .as_str()
        .expect("uri should be a string");

    assert!(uri.contains("verify=verify_peer"));
    assert!(uri.contains("cacertfile=/etc/ssl/certs/ca_bundle.pem"));
    assert!(uri.contains("certfile=/etc/ssl/certs/client.pem"));
    assert!(uri.contains("keyfile=/etc/ssl/private/client.key"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_enable_tls_peer_verification_for_all_upstreams_with_existing_params()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.modifications.test7";
    let upstream_name = "test_enable_existing_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_endpoint = format!("amqp://localhost:5672/{}", vh);
    let source_uri = format!(
        "{}?key1=abc&verify=verify_none&cacertfile=/old/path/ca.pem&key2=def&certfile=/old/path/client.pem&keyfile=/old/path/client.key&server_name_indication=example.com&custom_param=value123&another_param=xyz&heartbeat=60",
        amqp_endpoint
    );

    let mut args = vec![
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        upstream_name,
        "--uri",
        &source_uri,
        "--exchange-name",
        "x.fanout",
    ];
    if rabbitmq_version_is_at_least(3, 13, 0) {
        args.extend(["--queue-type", "classic"]);
    }
    run_succeeds(args);
    await_metric_emission(500);

    run_succeeds([
        "federation",
        "enable_tls_peer_verification_for_all_upstreams",
        "--node-local-ca-certificate-bundle-path",
        "/new/path/ca_bundle.pem",
        "--node-local-client-certificate-file-path",
        "/new/path/client.pem",
        "--node-local-client-private-key-file-path",
        "/new/path/client.key",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let upstream_param = params
        .iter()
        .find(|p| p.name == upstream_name && p.is_federation_upstream())
        .expect("Federation upstream parameter should exist");

    let uri = upstream_param.value["uri"]
        .as_str()
        .expect("uri should be a string");

    assert!(uri.contains("verify=verify_peer"));
    assert!(uri.contains("key1=abc"));
    assert!(uri.contains("key2=def"));
    assert!(uri.contains("cacertfile=/new/path/ca_bundle.pem"));
    assert!(uri.contains("certfile=/new/path/client.pem"));
    assert!(uri.contains("keyfile=/new/path/client.key"));
    assert!(uri.contains("server_name_indication=example.com"));
    assert!(uri.contains("custom_param=value123"));
    assert!(uri.contains("another_param=xyz"));
    assert!(uri.contains("heartbeat=60"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_enable_tls_peer_verification_for_all_upstreams_queue_federation()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.modifications.test8";
    let upstream_name = "test_enable_queue_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_endpoint = format!("amqp://localhost:5672/{}", vh);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        upstream_name,
        "--uri",
        &amqp_endpoint,
        "--queue-name",
        "test.queue",
        "--consumer-tag",
        "test-consumer",
    ]);

    run_succeeds([
        "federation",
        "enable_tls_peer_verification_for_all_upstreams",
        "--node-local-ca-certificate-bundle-path",
        "/etc/ssl/ca.pem",
        "--node-local-client-certificate-file-path",
        "/etc/ssl/client.pem",
        "--node-local-client-private-key-file-path",
        "/etc/ssl/client.key",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let upstream_param = params
        .iter()
        .find(|p| p.name == upstream_name && p.is_federation_upstream())
        .expect("Federation upstream parameter should exist");

    let uri = upstream_param.value["uri"]
        .as_str()
        .expect("uri should be a string");

    assert!(uri.contains("verify=verify_peer"));
    assert!(uri.contains("cacertfile=/etc/ssl/ca.pem"));
    assert!(uri.contains("certfile=/etc/ssl/client.pem"));
    assert!(uri.contains("keyfile=/etc/ssl/client.key"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_enable_tls_peer_verification_for_all_upstreams_mixed_federation()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.federation.modifications.test9";
    let exchange_upstream_name = "enable_exchange_upstream";
    let queue_upstream_name = "enable_queue_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_endpoint = format!("amqp://localhost:5672/{}", vh);
    let exchange_uri = format!(
        "{}?exchange_param=value1&verify=verify_none&old_cert=/old/path.pem",
        amqp_endpoint
    );
    let queue_uri = format!(
        "{}?queue_param=value2&verify=verify_none&old_key=/old/key.pem",
        amqp_endpoint
    );

    let mut args = vec![
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        exchange_upstream_name,
        "--uri",
        &exchange_uri,
        "--exchange-name",
        "x.federated",
    ];
    if rabbitmq_version_is_at_least(3, 13, 0) {
        args.extend(["--queue-type", "classic"]);
    }
    run_succeeds(args);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        queue_upstream_name,
        "--uri",
        &queue_uri,
        "--queue-name",
        "q.federated",
        "--consumer-tag",
        "mixed-consumer",
    ]);
    await_metric_emission(500);

    run_succeeds([
        "federation",
        "enable_tls_peer_verification_for_all_upstreams",
        "--node-local-ca-certificate-bundle-path",
        "/path/to/ca.pem",
        "--node-local-client-certificate-file-path",
        "/path/to/client.pem",
        "--node-local-client-private-key-file-path",
        "/path/to/client.key",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;

    let exchange_upstream_param = params
        .iter()
        .find(|p| p.name == exchange_upstream_name && p.is_federation_upstream())
        .expect("Exchange upstream parameter should exist");
    let queue_upstream_param = params
        .iter()
        .find(|p| p.name == queue_upstream_name && p.is_federation_upstream())
        .expect("Queue upstream parameter should exist");

    let exchange_uri = exchange_upstream_param.value["uri"]
        .as_str()
        .expect("uri should be a string");
    let queue_uri = queue_upstream_param.value["uri"]
        .as_str()
        .expect("uri should be a string");

    assert!(exchange_uri.contains("verify=verify_peer"));
    assert!(exchange_uri.contains("exchange_param=value1"));
    assert!(exchange_uri.contains("cacertfile=/path/to/ca.pem"));
    assert!(exchange_uri.contains("certfile=/path/to/client.pem"));
    assert!(exchange_uri.contains("keyfile=/path/to/client.key"));

    assert!(queue_uri.contains("verify=verify_peer"));
    assert!(queue_uri.contains("queue_param=value2"));
    assert!(queue_uri.contains("cacertfile=/path/to/ca.pem"));
    assert!(queue_uri.contains("certfile=/path/to/client.pem"));
    assert!(queue_uri.contains("keyfile=/path/to/client.key"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_upstreams_no_upstreams() -> Result<(), Box<dyn Error>>
{
    let vh = "rabbitmqadmin.federation.modifications.test_no_upstreams";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    run_succeeds([
        "federation",
        "disable_tls_peer_verification_for_all_upstreams",
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}
