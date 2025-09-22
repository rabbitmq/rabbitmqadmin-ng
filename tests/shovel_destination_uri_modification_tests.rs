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
use std::str;

#[test]
fn test_disable_tls_peer_verification_for_all_destination_uris_basic()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_destination_uris_basic";
    let shovel_name = "test_basic_dest_shovel";

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
        "disable_tls_peer_verification_for_all_destination_uris",
    ]);

    run_succeeds(["parameters", "list_all"])
        .stdout(predicate::str::contains(shovel_name))
        .stdout(predicate::str::contains("verify=verify_none"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_destination_uris_with_existing_verify_param()
-> Result<(), Box<dyn std::error::Error>> {
    let vh =
        "test_disable_tls_peer_verification_for_all_destination_uris_with_existing_verify_param";
    let shovel_name = "test_existing_dest_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_base = format!("amqps://localhost:5671/{}", vh);
    let source_uri = format!("{}?source_key=abc&heartbeat=60", amqps_base);
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
        "disable_tls_peer_verification_for_all_destination_uris",
    ]);

    let output = run_succeeds(["parameters", "list_all"]);
    let stdout = output
        .stdout(predicate::str::contains(shovel_name))
        .stdout(predicate::str::contains("verify=verify_none"))
        .stdout(predicate::str::contains("dest_key1=xyz"))
        .stdout(predicate::str::contains("dest_key2=abc"))
        .stdout(predicate::str::contains("cacertfile=/path/to/dest_ca.pem"))
        .stdout(predicate::str::contains(
            "certfile=/path/to/dest_client.pem",
        ))
        .stdout(predicate::str::contains("keyfile=/path/to/dest_client.key"))
        .stdout(predicate::str::contains(
            "server_name_indication=dest.example.com",
        ))
        .stdout(predicate::str::contains("dest_param=value456"))
        .stdout(predicate::str::contains("another_dest_param=def"))
        .stdout(predicate::str::contains("heartbeat=30"));

    let output_str = str::from_utf8(&stdout.get_output().stdout).unwrap();
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
            if line.contains("└─")
                || (in_our_shovel && line.contains("├─") && !line.contains(&shovel_name))
            {
                break;
            }
        }
    }

    let dest_uri_lines: Vec<&str> = shovel_section
        .lines()
        .filter(|line| line.contains("dest-uri"))
        .collect();
    assert!(
        !dest_uri_lines.is_empty(),
        "Could not find dest-uri in shovel section"
    );

    let dest_uri_content = dest_uri_lines[0];
    let dest_verify_count = dest_uri_content.matches("verify=").count();
    assert_eq!(
        dest_verify_count, 1,
        "Expected exactly 1 verify parameter in destination URI, found {}",
        dest_verify_count
    );

    assert!(
        dest_uri_content.contains("verify=verify_none"),
        "Destination URI should contain verify=verify_none"
    );
    assert!(
        !dest_uri_content.contains("verify=verify_peer"),
        "Destination URI should not contain verify=verify_peer"
    );

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_destination_uris_amqp10()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_destination_uris_amqp10";
    let shovel_name = "test_amqp10_dest_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_source = format!("amqps://localhost:5671/{}", vh);
    let amqps_destination = format!(
        "amqps://localhost:5671/{}?verify=verify_peer&certfile=/path/to/client.pem",
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
        "disable_tls_peer_verification_for_all_destination_uris",
    ]);

    run_succeeds(["parameters", "list_all"])
        .stdout(predicate::str::contains(shovel_name))
        .stdout(predicate::str::contains("verify=verify_none"))
        .stdout(predicate::str::contains("certfile=/path/to/client.pem"));

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_destination_uris_with_dummy_query_params()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_destination_uris_with_dummy_query_params";
    let shovel_name = "test_dummy_dest_params_shovel";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    let amqps_base = format!("amqps://localhost:5671/{}", vh);
    let source_uri = format!("{}?source_abc=123&source_heartbeat=5", amqps_base);
    let dest_uri = format!(
        "{}?dest_xyz=456&dest_heartbeat=10&channel_max=100&another_dummy=example",
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
        "disable_tls_peer_verification_for_all_destination_uris",
    ]);

    let output = run_succeeds(["parameters", "list_all"]);
    let stdout = output
        .stdout(predicate::str::contains(shovel_name))
        .stdout(predicate::str::contains("verify=verify_none"))
        .stdout(predicate::str::contains("dest_xyz=456"))
        .stdout(predicate::str::contains("dest_heartbeat=10"))
        .stdout(predicate::str::contains("channel_max=100"))
        .stdout(predicate::str::contains("another_dummy=example"));

    let output_str = str::from_utf8(&stdout.get_output().stdout).unwrap();
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
            if line.contains("└─")
                || (in_our_shovel && line.contains("├─") && !line.contains(&shovel_name))
            {
                break;
            }
        }
    }

    let dest_uri_lines: Vec<&str> = shovel_section
        .lines()
        .filter(|line| line.contains("dest-uri"))
        .collect();
    assert!(
        !dest_uri_lines.is_empty(),
        "Could not find dest-uri in shovel section"
    );

    let dest_uri_content = dest_uri_lines[0];
    let dest_verify_count = dest_uri_content.matches("verify=").count();
    assert_eq!(
        dest_verify_count, 1,
        "Expected exactly 1 verify parameter in destination URI, found {}",
        dest_verify_count
    );

    assert!(
        dest_uri_content.contains("verify=verify_none"),
        "Destination URI should contain verify=verify_none"
    );
    assert!(
        !dest_uri_content.contains("verify=verify_peer"),
        "Destination URI should not contain verify=verify_peer"
    );

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn test_disable_tls_peer_verification_for_all_destination_uris_no_shovels()
-> Result<(), Box<dyn std::error::Error>> {
    let vh = "test_disable_tls_peer_verification_for_all_destination_uris_no_shovels";

    delete_vhost(vh).ok();
    run_succeeds(["declare", "vhost", "--name", vh]);

    run_succeeds([
        "shovels",
        "disable_tls_peer_verification_for_all_destination_uris",
    ]);

    delete_vhost(vh).expect("failed to delete a virtual host");

    Ok(())
}
