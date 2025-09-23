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
use predicates::prelude::*;

#[test]
fn test_disable_tls_peer_verification_for_all_upstreams_basic()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_upstreams_basic";
    let upstream_name = "test_basic_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_endpoint = format!("amqps://localhost:5671/{}", vh);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        upstream_name,
        "--uri",
        &amqps_endpoint,
        "--exchange-name",
        "x.fanout",
        "--queue-type",
        "classic",
    ]);

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(upstream_name));

    run_succeeds([
        "federation",
        "disable_tls_peer_verification_for_all_upstreams",
    ]);

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(upstream_name))
        .stdout(predicate::str::contains("verify=verify_none"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_upstreams_with_existing_verify_param()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_upstreams_with_existing_verify_param";
    let upstream_name = "test_existing_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_endpoint = format!("amqps://localhost:5671/{}", vh);
    let source_uri = format!(
        "{}?key1=abc&verify=verify_peer&cacertfile=/path/to/ca_bundle.pem&key2=def&certfile=/path/to/client.pem&keyfile=/path/to/client.key&server_name_indication=example.com&custom_param=value123&another_param=xyz&heartbeat=60",
        amqps_endpoint
    );

    run_succeeds([
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
        "--queue-type",
        "classic",
    ]);
    await_metric_emission(500);

    run_succeeds([
        "federation",
        "disable_tls_peer_verification_for_all_upstreams",
    ]);

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(upstream_name))
        .stdout(predicate::str::contains("verify=verify_none"))
        .stdout(predicate::str::contains("key1=abc"))
        .stdout(predicate::str::contains("key2=def"))
        .stdout(predicate::str::contains(
            "cacertfile=/path/to/ca_bundle.pem",
        ))
        .stdout(predicate::str::contains("certfile=/path/to/client.pem"))
        .stdout(predicate::str::contains("keyfile=/path/to/client.key"))
        .stdout(predicate::str::contains(
            "server_name_indication=example.com",
        ))
        .stdout(predicate::str::contains("custom_param=value123"))
        .stdout(predicate::str::contains("another_param=xyz"))
        .stdout(predicate::str::contains("heartbeat=60"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_upstreams_queue_federation_basic()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_upstreams_queue_federation_basic";
    let upstream_name = "test_queue_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_endpoint = format!("amqps://localhost:5671/{}", vh);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        upstream_name,
        "--uri",
        &amqps_endpoint,
        "--queue-name",
        "test.queue",
        "--consumer-tag",
        "test-consumer",
    ]);

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(upstream_name));

    run_succeeds([
        "federation",
        "disable_tls_peer_verification_for_all_upstreams",
    ]);

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(upstream_name))
        .stdout(predicate::str::contains("verify=verify_none"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_upstreams_queue_federation_with_params()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_upstreams_queue_federation_with_params";
    let upstream_name = "test_queue_upstream_with_params";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_endpoint = format!("amqps://localhost:5671/{}", vh);
    let source_uri = format!(
        "{}?queue_param=test123&verify=verify_peer&cacertfile=/etc/ssl/certs/ca.pem&consumer_tag_param=custom&prefetch=100&ack_mode=on-confirm",
        amqps_endpoint
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

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(upstream_name))
        .stdout(predicate::str::contains("verify=verify_none"))
        .stdout(predicate::str::contains("queue_param=test123"))
        .stdout(predicate::str::contains("cacertfile=/etc/ssl/certs/ca.pem"))
        .stdout(predicate::str::contains("consumer_tag_param=custom"))
        .stdout(predicate::str::contains("prefetch=100"))
        .stdout(predicate::str::contains("ack_mode=on-confirm"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_upstreams_mixed_federation()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_upstreams_mixed_federation";
    let exchange_upstream_name = "exchange_upstream";
    let queue_upstream_name = "queue_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_endpoint = format!("amqps://localhost:5671/{}", vh);
    let exchange_uri = format!(
        "{}?exchange_param=value1&verify=verify_peer&certfile=/path/to/client.pem",
        amqps_endpoint
    );
    let queue_uri = format!(
        "{}?queue_param=value2&verify=verify_peer&keyfile=/path/to/client.key",
        amqps_endpoint
    );

    run_succeeds([
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
        "--queue-type",
        "classic",
    ]);

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

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(exchange_upstream_name))
        .stdout(predicate::str::contains(queue_upstream_name))
        .stdout(predicate::str::contains("exchange_param=value1"))
        .stdout(predicate::str::contains("queue_param=value2"))
        .stdout(predicate::str::contains("certfile=/path/to/client.pem"))
        .stdout(predicate::str::contains("keyfile=/path/to/client.key"))
        .stdout(predicate::str::contains("verify=verify_none"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_enable_tls_peer_verification_for_all_upstreams_basic()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_enable_tls_peer_verification_for_all_upstreams_basic";
    let upstream_name = "test_enable_basic_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_endpoint = format!("amqps://localhost:5671/{}", vh);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        upstream_name,
        "--uri",
        &amqps_endpoint,
        "--exchange-name",
        "x.fanout",
        "--queue-type",
        "classic",
    ]);

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(upstream_name));

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

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(upstream_name))
        .stdout(predicate::str::contains("verify=verify_peer"))
        .stdout(predicate::str::contains(
            "cacertfile=/etc/ssl/certs/ca_bundle.pem",
        ))
        .stdout(predicate::str::contains(
            "certfile=/etc/ssl/certs/client.pem",
        ))
        .stdout(predicate::str::contains(
            "keyfile=/etc/ssl/private/client.key",
        ));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_enable_tls_peer_verification_for_all_upstreams_with_existing_params()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_enable_tls_peer_verification_for_all_upstreams_with_existing_params";
    let upstream_name = "test_enable_existing_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_endpoint = format!("amqps://localhost:5671/{}", vh);
    let source_uri = format!(
        "{}?key1=abc&verify=verify_none&cacertfile=/old/path/ca.pem&key2=def&certfile=/old/path/client.pem&keyfile=/old/path/client.key&server_name_indication=example.com&custom_param=value123&another_param=xyz&heartbeat=60",
        amqps_endpoint
    );

    run_succeeds([
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
        "--queue-type",
        "classic",
    ]);
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

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(upstream_name))
        .stdout(predicate::str::contains("verify=verify_peer"))
        .stdout(predicate::str::contains("key1=abc"))
        .stdout(predicate::str::contains("key2=def"))
        .stdout(predicate::str::contains(
            "cacertfile=/new/path/ca_bundle.pem",
        ))
        .stdout(predicate::str::contains("certfile=/new/path/client.pem"))
        .stdout(predicate::str::contains("keyfile=/new/path/client.key"))
        .stdout(predicate::str::contains(
            "server_name_indication=example.com",
        ))
        .stdout(predicate::str::contains("custom_param=value123"))
        .stdout(predicate::str::contains("another_param=xyz"))
        .stdout(predicate::str::contains("heartbeat=60"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_enable_tls_peer_verification_for_all_upstreams_queue_federation()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_enable_tls_peer_verification_for_all_upstreams_queue_federation";
    let upstream_name = "test_enable_queue_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_endpoint = format!("amqps://localhost:5671/{}", vh);

    run_succeeds([
        "-V",
        vh,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        upstream_name,
        "--uri",
        &amqps_endpoint,
        "--queue-name",
        "test.queue",
        "--consumer-tag",
        "test-consumer",
    ]);

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(upstream_name));

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

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(upstream_name))
        .stdout(predicate::str::contains("verify=verify_peer"))
        .stdout(predicate::str::contains("cacertfile=/etc/ssl/ca.pem"))
        .stdout(predicate::str::contains("certfile=/etc/ssl/client.pem"))
        .stdout(predicate::str::contains("keyfile=/etc/ssl/client.key"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_enable_tls_peer_verification_for_all_upstreams_mixed_federation()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_enable_tls_peer_verification_for_all_upstreams_mixed_federation";
    let exchange_upstream_name = "enable_exchange_upstream";
    let queue_upstream_name = "enable_queue_upstream";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_endpoint = format!("amqps://localhost:5671/{}", vh);
    let exchange_uri = format!(
        "{}?exchange_param=value1&verify=verify_none&old_cert=/old/path.pem",
        amqps_endpoint
    );
    let queue_uri = format!(
        "{}?queue_param=value2&verify=verify_none&old_key=/old/key.pem",
        amqps_endpoint
    );

    run_succeeds([
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
        "--queue-type",
        "classic",
    ]);

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

    run_succeeds(["federation", "list_all_upstreams"])
        .stdout(predicate::str::contains(exchange_upstream_name))
        .stdout(predicate::str::contains(queue_upstream_name))
        .stdout(predicate::str::contains("exchange_param=value1"))
        .stdout(predicate::str::contains("queue_param=value2"))
        .stdout(predicate::str::contains("cacertfile=/path/to/ca.pem"))
        .stdout(predicate::str::contains("certfile=/path/to/client.pem"))
        .stdout(predicate::str::contains("keyfile=/path/to/client.key"))
        .stdout(predicate::str::contains("verify=verify_peer"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}
