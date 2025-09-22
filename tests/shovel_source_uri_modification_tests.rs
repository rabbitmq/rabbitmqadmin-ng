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
fn test_disable_tls_peer_verification_for_all_shovels_basic() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_shovels_basic";
    let shovel_name = "test_basic_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_source = format!("amqps://localhost:5671/{}", vh);
    let amqps_destination = format!("amqps://localhost:5671/{}", vh);

    run_succeeds([
        "-V",
        vh,
        "shovels",
        "declare_amqp091",
        "--name",
        shovel_name,
        "--source-uri",
        &amqps_source,
        "--destination-uri",
        &amqps_destination,
        "--source-queue",
        "source.queue",
        "--destination-queue",
        "dest.queue",
        "--ack-mode",
        "on-confirm",
    ]);

    run_succeeds(["parameters", "list_all"]).stdout(predicate::str::contains(shovel_name));

    run_succeeds([
        "shovels",
        "disable_tls_peer_verification_for_all_source_uris",
    ]);

    run_succeeds(["parameters", "list_all"])
        .stdout(predicate::str::contains(shovel_name))
        .stdout(predicate::str::contains("verify=verify_none"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_shovels_with_existing_verify_param()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_shovels_with_existing_verify_param";
    let shovel_name = "test_existing_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_base = format!("amqps://localhost:5671/{}", vh);
    let source_uri = format!(
        "{}?key1=abc&verify=verify_peer&cacertfile=/path/to/ca_bundle.pem&key2=def&certfile=/path/to/client.pem&keyfile=/path/to/client.key&server_name_indication=example.com&custom_param=value123&another_param=xyz&heartbeat=60",
        amqps_base
    );
    let dest_uri = format!(
        "{}?dest_key1=xyz&verify=verify_peer&cacertfile=/path/to/dest_ca.pem&dest_key2=abc&certfile=/path/to/dest_client.pem&keyfile=/path/to/dest_client.key&server_name_indication=dest.example.com&dest_param=value456&another_dest_param=def&heartbeat=30",
        amqps_base
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

    let output = run_succeeds(["parameters", "list_all"]);
    let stdout = output.stdout(predicate::str::contains(shovel_name))
        .stdout(predicate::str::contains("verify=verify_none"))
        .stdout(predicate::str::contains("key1=abc"))
        .stdout(predicate::str::contains("key2=def"))
        .stdout(predicate::str::contains("cacertfile=/path/to/ca_bundle.pem"))
        .stdout(predicate::str::contains("certfile=/path/to/client.pem"))
        .stdout(predicate::str::contains("keyfile=/path/to/client.key"))
        .stdout(predicate::str::contains("server_name_indication=example.com"))
        .stdout(predicate::str::contains("custom_param=value123"))
        .stdout(predicate::str::contains("another_param=xyz"))
        .stdout(predicate::str::contains("heartbeat=60"))
        .stdout(predicate::str::contains("dest_key1=xyz"))
        .stdout(predicate::str::contains("dest_key2=abc"))
        .stdout(predicate::str::contains("cacertfile=/path/to/dest_ca.pem"))
        .stdout(predicate::str::contains("certfile=/path/to/dest_client.pem"))
        .stdout(predicate::str::contains("keyfile=/path/to/dest_client.key"))
        .stdout(predicate::str::contains("server_name_indication=dest.example.com"))
        .stdout(predicate::str::contains("dest_param=value456"))
        .stdout(predicate::str::contains("another_dest_param=def"))
        .stdout(predicate::str::contains("heartbeat=30"));

    let output_str = std::str::from_utf8(&stdout.get_output().stdout).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();
    let mut shovel_section = String::new();
    let mut in_our_shovel = false;

    for line in lines {
        if line.contains(&shovel_name) {
            in_our_shovel = true;
        }
        if in_our_shovel {
            shovel_section.push_str(line);
            shovel_section.push('\n');
            if line.contains("└─") || (in_our_shovel && line.contains("├─") && !line.contains(&shovel_name)) {
                break;
            }
        }
    }

    let src_uri_lines: Vec<&str> = shovel_section.lines()
        .filter(|line| line.contains("src-uri"))
        .collect();
    assert!(!src_uri_lines.is_empty(), "Could not find src-uri in shovel section");

    let src_uri_content = src_uri_lines[0];
    let src_verify_count = src_uri_content.matches("verify=").count();
    assert_eq!(src_verify_count, 1, "Expected exactly 1 verify parameter in source URI, found {}", src_verify_count);

    assert!(src_uri_content.contains("verify=verify_none"), "Source URI should contain verify=verify_none");
    assert!(!src_uri_content.contains("verify=verify_peer"), "Source URI should not contain verify=verify_peer");

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_shovels_amqp10() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_shovels_amqp10";
    let shovel_name = "test_amqp10_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_source = format!("amqps://localhost:5671/{}?verify=verify_peer&cacertfile=/path/to/ca.pem", vh);
    let amqps_destination = format!("amqps://localhost:5671/{}?verify=verify_peer&certfile=/path/to/client.pem", vh);

    run_succeeds([
        "-V",
        vh,
        "shovels",
        "declare_amqp10",
        "--name",
        shovel_name,
        "--source-uri",
        &amqps_source,
        "--destination-uri",
        &amqps_destination,
        "--source-address",
        "source.address",
        "--destination-address",
        "dest.address",
        "--ack-mode",
        "on-confirm",
    ]);

    run_succeeds(["parameters", "list_all"]).stdout(predicate::str::contains(shovel_name));

    run_succeeds([
        "shovels",
        "disable_tls_peer_verification_for_all_source_uris",
    ]);

    run_succeeds(["parameters", "list_all"])
        .stdout(predicate::str::contains(shovel_name))
        .stdout(predicate::str::contains("verify=verify_none"))
        .stdout(predicate::str::contains("cacertfile=/path/to/ca.pem"))
        .stdout(predicate::str::contains("certfile=/path/to/client.pem"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_shovels_mixed_protocols()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_shovels_mixed_protocols";
    let shovel_091_name = "test_091_shovel";
    let shovel_10_name = "test_10_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_base = format!("amqps://localhost:5671/{}", vh);
    let uri_091_source = format!("{}?protocol_param=091&verify=verify_peer&certfile=/path/to/091.pem", amqps_base);
    let uri_091_dest = format!("{}?protocol_param=091_dest&verify=verify_peer&keyfile=/path/to/091.key", amqps_base);
    let uri_10_source = format!("{}?protocol_param=10&verify=verify_peer&cacertfile=/path/to/10.pem", amqps_base);
    let uri_10_dest = format!("{}?protocol_param=10_dest&verify=verify_peer&server_name_indication=amqp10.example.com", amqps_base);

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

    run_succeeds(["parameters", "list_all"])
        .stdout(predicate::str::contains(shovel_091_name))
        .stdout(predicate::str::contains(shovel_10_name))
        .stdout(predicate::str::contains("protocol_param=091"))
        .stdout(predicate::str::contains("protocol_param=091_dest"))
        .stdout(predicate::str::contains("protocol_param=10"))
        .stdout(predicate::str::contains("protocol_param=10_dest"))
        .stdout(predicate::str::contains("certfile=/path/to/091.pem"))
        .stdout(predicate::str::contains("keyfile=/path/to/091.key"))
        .stdout(predicate::str::contains("cacertfile=/path/to/10.pem"))
        .stdout(predicate::str::contains("server_name_indication=amqp10.example.com"))
        .stdout(predicate::str::contains("verify=verify_none"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_shovels_no_shovels()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_shovels_no_shovels";

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
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_shovels_with_dummy_query_params";
    let shovel_name = "test_dummy_params_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_base = format!("amqps://localhost:5671/{}", vh);
    let source_uri = format!("{}?abc=123&heartbeat=5&connection_timeout=30&dummy_param=test_value", amqps_base);
    let dest_uri = format!("{}?xyz=456&heartbeat=10&channel_max=100&another_dummy=example", amqps_base);

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

    let output = run_succeeds(["parameters", "list_all"]);
    let stdout = output.stdout(predicate::str::contains(shovel_name))
        .stdout(predicate::str::contains("verify=verify_none"))
        .stdout(predicate::str::contains("abc=123"))
        .stdout(predicate::str::contains("heartbeat=5"))
        .stdout(predicate::str::contains("connection_timeout=30"))
        .stdout(predicate::str::contains("dummy_param=test_value"))
        .stdout(predicate::str::contains("xyz=456"))
        .stdout(predicate::str::contains("heartbeat=10"))
        .stdout(predicate::str::contains("channel_max=100"))
        .stdout(predicate::str::contains("another_dummy=example"));

    let output_str = std::str::from_utf8(&stdout.get_output().stdout).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();
    let mut shovel_section = String::new();
    let mut in_our_shovel = false;

    for line in lines {
        if line.contains(&shovel_name) {
            in_our_shovel = true;
        }
        if in_our_shovel {
            shovel_section.push_str(line);
            shovel_section.push('\n');
            if line.contains("└─") || (in_our_shovel && line.contains("├─") && !line.contains(&shovel_name)) {
                break;
            }
        }
    }

    let src_uri_lines: Vec<&str> = shovel_section.lines()
        .filter(|line| line.contains("src-uri"))
        .collect();
    assert!(!src_uri_lines.is_empty(), "Could not find src-uri in shovel section");

    let src_uri_content = src_uri_lines[0];
    let src_verify_count = src_uri_content.matches("verify=").count();
    assert_eq!(src_verify_count, 1, "Expected exactly 1 verify parameter in source URI, found {}", src_verify_count);

    assert!(src_uri_content.contains("verify=verify_none"), "Source URI should contain verify=verify_none");
    assert!(!src_uri_content.contains("verify=verify_peer"), "Source URI should not contain verify=verify_peer");

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}