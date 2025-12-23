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
fn test_disable_tls_peer_verification_for_all_shovels_basic() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.shovel.modifications.test1";
    let shovel_name = "test_basic_shovel";

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
        "disable_tls_peer_verification_for_all_source_uris",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let shovel_param = params
        .iter()
        .find(|p| p.name == shovel_name && p.is_shovel())
        .expect("Shovel parameter should exist");

    let source_uri = shovel_param.value["src-uri"]
        .as_str()
        .expect("src-uri should be a string");
    assert!(source_uri.contains("verify=verify_none"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_shovels_with_existing_verify_param()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.shovel.modifications.test2";
    let shovel_name = "test_existing_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_base = format!("amqp://localhost:5672/{}", vh);
    let source_uri = format!(
        "{}?key1=abc&verify=verify_peer&cacertfile=/path/to/ca_bundle.pem&key2=def&certfile=/path/to/client.pem&keyfile=/path/to/client.key&server_name_indication=example.com&custom_param=value123&another_param=xyz&heartbeat=60",
        amqp_base
    );
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
        "disable_tls_peer_verification_for_all_source_uris",
    ]);

    let client = api_client();
    let params_after = client.list_runtime_parameters()?;
    let shovel_param = params_after
        .iter()
        .find(|p| p.name == shovel_name && p.is_shovel())
        .expect("Shovel parameter should exist");

    let source_uri_after = shovel_param.value["src-uri"]
        .as_str()
        .expect("src-uri should be a string");
    let dest_uri_after = shovel_param.value["dest-uri"]
        .as_str()
        .expect("dest-uri should be a string");

    // Check that source URI has verify=verify_none and preserves other parameters
    assert!(source_uri_after.contains("verify=verify_none"));
    assert!(!source_uri_after.contains("verify=verify_peer"));
    assert!(source_uri_after.contains("key1=abc"));
    assert!(source_uri_after.contains("key2=def"));
    assert!(source_uri_after.contains("cacertfile=/path/to/ca_bundle.pem"));
    assert!(source_uri_after.contains("certfile=/path/to/client.pem"));
    assert!(source_uri_after.contains("keyfile=/path/to/client.key"));
    assert!(source_uri_after.contains("server_name_indication=example.com"));
    assert!(source_uri_after.contains("custom_param=value123"));
    assert!(source_uri_after.contains("another_param=xyz"));
    assert!(source_uri_after.contains("heartbeat=60"));

    // Check that destination URI is unchanged
    assert!(dest_uri_after.contains("verify=verify_peer"));
    assert!(dest_uri_after.contains("dest_key1=xyz"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_shovels_amqp10() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.shovel.modifications.test3";
    let shovel_name = "test_amqp10_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_source = format!(
        "amqp://localhost:5672/{}?verify=verify_peer&cacertfile=/path/to/ca.pem",
        vh
    );
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
        "disable_tls_peer_verification_for_all_source_uris",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;
    let shovel_param = params
        .iter()
        .find(|p| p.name == shovel_name && p.is_shovel())
        .expect("Shovel parameter should exist");

    let source_uri = shovel_param.value["src-uri"]
        .as_str()
        .expect("src-uri should be a string");
    let dest_uri = shovel_param.value["dest-uri"]
        .as_str()
        .expect("dest-uri should be a string");

    assert!(source_uri.contains("verify=verify_none"));
    assert!(source_uri.contains("cacertfile=/path/to/ca.pem"));
    assert!(dest_uri.contains("certfile=/path/to/client.pem"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_shovels_mixed_protocols() -> Result<(), Box<dyn Error>>
{
    let vh = "rabbitmqadmin.shovel.modifications.test4";
    let shovel_091_name = "test_091_shovel";
    let shovel_10_name = "test_10_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_base = format!("amqp://localhost:5672/{}", vh);
    let uri_091_source = format!(
        "{}?protocol_param=091&verify=verify_peer&certfile=/path/to/091.pem",
        amqp_base
    );
    let uri_091_dest = format!(
        "{}?protocol_param=091_dest&verify=verify_peer&keyfile=/path/to/091.key",
        amqp_base
    );
    let uri_10_source = format!(
        "{}?protocol_param=10&verify=verify_peer&cacertfile=/path/to/10.pem",
        amqp_base
    );
    let uri_10_dest = format!(
        "{}?protocol_param=10_dest&verify=verify_peer&server_name_indication=amqp10.example.com",
        amqp_base
    );

    run_succeeds([
        "-V",
        vh,
        "shovels",
        "declare_amqp091",
        "--name",
        shovel_091_name,
        "--source-uri",
        &uri_091_source,
        "--destination-uri",
        &uri_091_dest,
        "--source-queue",
        "q.091.source",
        "--destination-queue",
        "q.091.dest",
        "--ack-mode",
        "on-confirm",
    ]);

    run_succeeds([
        "-V",
        vh,
        "shovels",
        "declare_amqp10",
        "--name",
        shovel_10_name,
        "--source-uri",
        &uri_10_source,
        "--destination-uri",
        &uri_10_dest,
        "--source-address",
        "addr.10.source",
        "--destination-address",
        "addr.10.dest",
        "--ack-mode",
        "on-confirm",
    ]);
    await_metric_emission(500);

    run_succeeds([
        "shovels",
        "disable_tls_peer_verification_for_all_source_uris",
    ]);

    let client = api_client();
    let params = client.list_runtime_parameters()?;

    let shovel_091_param = params
        .iter()
        .find(|p| p.name == shovel_091_name && p.is_shovel())
        .expect("091 shovel parameter should exist");
    let shovel_10_param = params
        .iter()
        .find(|p| p.name == shovel_10_name && p.is_shovel())
        .expect("10 shovel parameter should exist");

    let uri_091_src = shovel_091_param.value["src-uri"]
        .as_str()
        .expect("src-uri should be a string");
    let uri_10_src = shovel_10_param.value["src-uri"]
        .as_str()
        .expect("src-uri should be a string");

    assert!(uri_091_src.contains("verify=verify_none"));
    assert!(uri_091_src.contains("protocol_param=091"));
    assert!(uri_091_src.contains("certfile=/path/to/091.pem"));

    assert!(uri_10_src.contains("verify=verify_none"));
    assert!(uri_10_src.contains("protocol_param=10"));
    assert!(uri_10_src.contains("cacertfile=/path/to/10.pem"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_shovels_no_shovels() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.shovel.modifications.test5";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    run_succeeds([
        "shovels",
        "disable_tls_peer_verification_for_all_source_uris",
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_shovels_with_dummy_query_params()
-> Result<(), Box<dyn Error>> {
    let vh =
        "rabbitmqadmin.test_disable_tls_peer_verification_for_all_shovels_with_dummy_query_params";
    let shovel_name = "test_dummy_params_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_base = format!("amqp://localhost:5672/{}", vh);
    let source_uri = format!(
        "{}?abc=123&heartbeat=5&connection_timeout=30&dummy_param=test_value",
        amqp_base
    );
    let dest_uri = format!(
        "{}?xyz=456&heartbeat=10&channel_max=100&another_dummy=example",
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
        "disable_tls_peer_verification_for_all_source_uris",
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

    assert!(source_uri_after.contains("verify=verify_none"));
    assert!(source_uri_after.contains("abc=123"));
    assert!(source_uri_after.contains("heartbeat=5"));
    assert!(source_uri_after.contains("connection_timeout=30"));
    assert!(source_uri_after.contains("dummy_param=test_value"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_enable_tls_peer_verification_for_all_source_uris_basic() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.shovel.modifications.test6";
    let shovel_name = "test_enable_basic_shovel";

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
        "enable_tls_peer_verification_for_all_source_uris",
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

    let source_uri = shovel_param.value["src-uri"]
        .as_str()
        .expect("src-uri should be a string");

    assert!(source_uri.contains("verify=verify_peer"));
    assert!(source_uri.contains("cacertfile=/etc/ssl/certs/ca_bundle.pem"));
    assert!(source_uri.contains("certfile=/etc/ssl/certs/client.pem"));
    assert!(source_uri.contains("keyfile=/etc/ssl/private/client.key"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_enable_tls_peer_verification_for_all_source_uris_with_existing_params()
-> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.shovel.modifications.test7";
    let shovel_name = "test_enable_existing_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqp_base = format!("amqp://localhost:5672/{}", vh);
    let source_uri = format!(
        "{}?key1=abc&verify=verify_none&cacertfile=/old/path/ca.pem&key2=def&certfile=/old/path/client.pem&keyfile=/old/path/client.key&server_name_indication=example.com&custom_param=value123&another_param=xyz&heartbeat=60",
        amqp_base
    );
    let destination_uri = format!("amqp://localhost:5672/{}", vh);

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
        "enable_tls_peer_verification_for_all_source_uris",
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

    let source_uri_after = shovel_param.value["src-uri"]
        .as_str()
        .expect("src-uri should be a string");

    assert!(source_uri_after.contains("verify=verify_peer"));
    assert!(source_uri_after.contains("cacertfile=/etc/ssl/certs/ca_bundle.pem"));
    assert!(source_uri_after.contains("certfile=/etc/ssl/certs/client.pem"));
    assert!(source_uri_after.contains("keyfile=/etc/ssl/private/client.key"));
    assert!(!source_uri_after.contains("cacertfile=/old/path/ca.pem"));
    assert!(!source_uri_after.contains("certfile=/old/path/client.pem"));
    assert!(!source_uri_after.contains("keyfile=/old/path/client.key"));
    assert!(source_uri_after.contains("key1=abc"));
    assert!(source_uri_after.contains("key2=def"));
    assert!(source_uri_after.contains("server_name_indication=example.com"));
    assert!(source_uri_after.contains("custom_param=value123"));
    assert!(source_uri_after.contains("another_param=xyz"));
    assert!(source_uri_after.contains("heartbeat=60"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}
