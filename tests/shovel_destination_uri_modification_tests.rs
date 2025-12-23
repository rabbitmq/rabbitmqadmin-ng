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
use std::error::Error;
#[test]
fn test_disable_tls_peer_verification_for_all_destination_uris_basic() -> Result<(), Box<dyn Error>>
{
    let vh = "rabbitmqadmin.shovel.modifications.test8";
    let shovel_name = "test_basic_dest_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_source = format!("amqp://localhost:5672/{}", vh);
    let amqp_destination = format!("amqp://localhost:5672/{}", vh);

    run_succeeds([
        "-V",
        vh,
        "shovels",
        "declare_amqp091",
        "--name",
        shovel_name,
        "--source-uri",
        &amqp_source,
        "--destination-uri",
        &amqp_destination,
        "--source-queue",
        "source.queue",
        "--destination-queue",
        "dest.queue",
        "--ack-mode",
        "on-confirm",
    ]);

    run_succeeds([
        "shovels",
        "disable_tls_peer_verification_for_all_destination_uris",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let shovel_param = params
        .iter()
        .find(|p| p.name == shovel_name && p.is_shovel())
        .expect("Shovel parameter should exist");

    let dest_uri = shovel_param.value["dest-uri"]
        .as_str()
        .expect("dest-uri should be a string");
    assert!(dest_uri.contains("verify=verify_none"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_destination_uris_with_existing_verify_param()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.shovel.modifications.test9";
    let shovel_name = "test_existing_dest_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_base = format!("amqp://localhost:5672/{}", vh);
    let source_uri = format!("{}?source_key=abc&heartbeat=60", amqp_base);
    let dest_uri = format!(
        "{}?dest_key1=xyz&verify=verify_peer&cacertfile=/path/to/dest_ca.pem&dest_key2=abc&certfile=/path/to/dest_client.pem&keyfile=/path/to/dest_client.key&server_name_indication=dest.example.com&dest_param=value456&another_dest_param=def&heartbeat=30",
        amqp_base
    );

    run_succeeds([
        "-V",
        vh,
        "shovels",
        "declare_amqp091",
        "--name",
        shovel_name,
        "--source-uri",
        &source_uri,
        "--destination-uri",
        &dest_uri,
        "--source-queue",
        "source.queue",
        "--destination-queue",
        "dest.queue",
        "--ack-mode",
        "on-confirm",
    ]);
    await_metric_emission(500);

    run_succeeds([
        "shovels",
        "disable_tls_peer_verification_for_all_destination_uris",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let shovel_param = params
        .iter()
        .find(|p| p.name == shovel_name && p.is_shovel())
        .expect("Shovel parameter should exist");

    let source_uri_after = shovel_param.value["src-uri"]
        .as_str()
        .expect("src-uri should be a string");
    let dest_uri_after = shovel_param.value["dest-uri"]
        .as_str()
        .expect("dest-uri should be a string");

    // Check that destination URI has verify=verify_none and preserves other parameters
    assert!(dest_uri_after.contains("verify=verify_none"));
    assert!(!dest_uri_after.contains("verify=verify_peer"));
    assert!(dest_uri_after.contains("dest_key1=xyz"));
    assert!(dest_uri_after.contains("dest_key2=abc"));
    assert!(dest_uri_after.contains("cacertfile=/path/to/dest_ca.pem"));
    assert!(dest_uri_after.contains("certfile=/path/to/dest_client.pem"));
    assert!(dest_uri_after.contains("keyfile=/path/to/dest_client.key"));
    assert!(dest_uri_after.contains("server_name_indication=dest.example.com"));
    assert!(dest_uri_after.contains("dest_param=value456"));
    assert!(dest_uri_after.contains("another_dest_param=def"));
    assert!(dest_uri_after.contains("heartbeat=30"));

    // Check that source URI is unchanged
    assert!(source_uri_after.contains("source_key=abc"));
    assert!(source_uri_after.contains("heartbeat=60"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_destination_uris_amqp10() -> Result<(), Box<dyn Error>>
{
    let vh = "rabbitmqadmin.shovel.modifications.test10";
    let shovel_name = "test_amqp10_dest_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_source = format!("amqp://localhost:5672/{}", vh);
    let amqp_destination = format!(
        "amqp://localhost:5672/{}?verify=verify_peer&certfile=/path/to/client.pem",
        vh
    );

    run_succeeds([
        "-V",
        vh,
        "shovels",
        "declare_amqp10",
        "--name",
        shovel_name,
        "--source-uri",
        &amqp_source,
        "--destination-uri",
        &amqp_destination,
        "--source-address",
        "source.address",
        "--destination-address",
        "dest.address",
        "--ack-mode",
        "on-confirm",
    ]);

    run_succeeds([
        "shovels",
        "disable_tls_peer_verification_for_all_destination_uris",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let shovel_param = params
        .iter()
        .find(|p| p.name == shovel_name && p.is_shovel())
        .expect("Shovel parameter should exist");

    let dest_uri = shovel_param.value["dest-uri"]
        .as_str()
        .expect("dest-uri should be a string");

    assert!(dest_uri.contains("verify=verify_none"));
    assert!(dest_uri.contains("certfile=/path/to/client.pem"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_destination_uris_with_dummy_query_params()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.shovel.modifications.test11";
    let shovel_name = "test_dummy_dest_params_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_base = format!("amqp://localhost:5672/{}", vh);
    let source_uri = format!("{}?source_abc=123&source_heartbeat=5", amqp_base);
    let dest_uri = format!(
        "{}?dest_xyz=456&dest_heartbeat=10&channel_max=100&another_dummy=example",
        amqp_base
    );

    run_succeeds([
        "-V",
        vh,
        "shovels",
        "declare_amqp091",
        "--name",
        shovel_name,
        "--source-uri",
        &source_uri,
        "--destination-uri",
        &dest_uri,
        "--source-queue",
        "source.queue",
        "--destination-queue",
        "dest.queue",
        "--ack-mode",
        "on-confirm",
    ]);
    await_metric_emission(500);

    run_succeeds([
        "shovels",
        "disable_tls_peer_verification_for_all_destination_uris",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let shovel_param = params
        .iter()
        .find(|p| p.name == shovel_name && p.is_shovel())
        .expect("Shovel parameter should exist");

    let dest_uri_after = shovel_param.value["dest-uri"]
        .as_str()
        .expect("dest-uri should be a string");

    assert!(dest_uri_after.contains("verify=verify_none"));
    assert!(dest_uri_after.contains("dest_xyz=456"));
    assert!(dest_uri_after.contains("dest_heartbeat=10"));
    assert!(dest_uri_after.contains("channel_max=100"));
    assert!(dest_uri_after.contains("another_dummy=example"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_destination_uris_no_shovels()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.shovel.modifications.test12";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    run_succeeds([
        "shovels",
        "disable_tls_peer_verification_for_all_destination_uris",
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_enable_tls_peer_verification_for_all_destination_uris_basic() -> Result<(), Box<dyn Error>>
{
    let vh = "rabbitmqadmin.shovel.modifications.test13";
    let shovel_name = "test_enable_basic_dest_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_source = format!("amqp://localhost:5672/{}", vh);
    let amqp_destination = format!("amqp://localhost:5672/{}", vh);

    run_succeeds([
        "-V",
        vh,
        "shovels",
        "declare_amqp091",
        "--name",
        shovel_name,
        "--source-uri",
        &amqp_source,
        "--destination-uri",
        &amqp_destination,
        "--source-queue",
        "source.queue",
        "--destination-queue",
        "dest.queue",
        "--ack-mode",
        "on-confirm",
    ]);

    run_succeeds([
        "shovels",
        "enable_tls_peer_verification_for_all_destination_uris",
        "--node-local-ca-certificate-bundle-path",
        "/etc/ssl/certs/ca_bundle.pem",
        "--node-local-client-certificate-file-path",
        "/etc/ssl/certs/client.pem",
        "--node-local-client-private-key-file-path",
        "/etc/ssl/private/client.key",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let shovel_param = params
        .iter()
        .find(|p| p.name == shovel_name && p.is_shovel())
        .expect("Shovel parameter should exist");

    let dest_uri = shovel_param.value["dest-uri"]
        .as_str()
        .expect("dest-uri should be a string");

    assert!(dest_uri.contains("verify=verify_peer"));
    assert!(dest_uri.contains("cacertfile=/etc/ssl/certs/ca_bundle.pem"));
    assert!(dest_uri.contains("certfile=/etc/ssl/certs/client.pem"));
    assert!(dest_uri.contains("keyfile=/etc/ssl/private/client.key"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_enable_tls_peer_verification_for_all_destination_uris_with_existing_params()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.shovel.modifications.test14";
    let shovel_name = "test_enable_existing_dest_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_base = format!("amqp://localhost:5672/{}", vh);
    let source_uri = format!("amqp://localhost:5672/{}", vh);
    let destination_uri = format!(
        "{}?key1=abc&verify=verify_none&cacertfile=/old/path/ca.pem&key2=def&certfile=/old/path/client.pem&keyfile=/old/path/client.key&server_name_indication=example.com&custom_param=value123&another_param=xyz&heartbeat=60",
        amqp_base
    );

    run_succeeds([
        "-V",
        vh,
        "shovels",
        "declare_amqp091",
        "--name",
        shovel_name,
        "--source-uri",
        &source_uri,
        "--destination-uri",
        &destination_uri,
        "--source-queue",
        "source.queue",
        "--destination-queue",
        "dest.queue",
        "--ack-mode",
        "on-confirm",
    ]);
    await_metric_emission(500);

    run_succeeds([
        "shovels",
        "enable_tls_peer_verification_for_all_destination_uris",
        "--node-local-ca-certificate-bundle-path",
        "/etc/ssl/certs/ca_bundle.pem",
        "--node-local-client-certificate-file-path",
        "/etc/ssl/certs/client.pem",
        "--node-local-client-private-key-file-path",
        "/etc/ssl/private/client.key",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let shovel_param = params
        .iter()
        .find(|p| p.name == shovel_name && p.is_shovel())
        .expect("Shovel parameter should exist");

    let dest_uri2 = shovel_param.value["dest-uri"]
        .as_str()
        .expect("dest-uri should be a string");

    assert!(dest_uri2.contains("verify=verify_peer"));
    assert!(dest_uri2.contains("cacertfile=/etc/ssl/certs/ca_bundle.pem"));
    assert!(dest_uri2.contains("certfile=/etc/ssl/certs/client.pem"));
    assert!(dest_uri2.contains("keyfile=/etc/ssl/private/client.key"));
    assert!(!dest_uri2.contains("cacertfile=/old/path/ca.pem"));
    assert!(!dest_uri2.contains("certfile=/old/path/client.pem"));
    assert!(!dest_uri2.contains("keyfile=/old/path/client.key"));
    assert!(dest_uri2.contains("key1=abc"));
    assert!(dest_uri2.contains("key2=def"));
    assert!(dest_uri2.contains("server_name_indication=example.com"));
    assert!(dest_uri2.contains("custom_param=value123"));
    assert!(dest_uri2.contains("another_param=xyz"));
    assert!(dest_uri2.contains("heartbeat=60"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}
