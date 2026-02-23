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

use std::error::Error;

use crate::test_helpers::{
    amqp_endpoint_with_vhost, await_federation_link_with, await_ms, await_no_federation_link_with,
    delete_vhost, run_succeeds,
};

#[test]
fn test_federation_link_stops_after_policy_disables_federation() -> Result<(), Box<dyn Error>> {
    let vh_a = "rabbitmqadmin.federation.link_term.1a";
    let vh_b = "rabbitmqadmin.federation.link_term.1b";
    let upstream_name = "up.link_term.policy_disable";
    let policy_name = "pol.link_term.policy_disable";
    let q = "federation.link_term.1";
    let amqp_uri = amqp_endpoint_with_vhost(vh_b);

    delete_vhost(vh_a).ok();
    delete_vhost(vh_b).ok();
    run_succeeds(["declare", "vhost", "--name", vh_a]);
    run_succeeds(["declare", "vhost", "--name", vh_b]);

    run_succeeds([
        "-V",
        vh_a,
        "federation",
        "declare_upstream",
        "--name",
        upstream_name,
        "--uri",
        &amqp_uri,
    ]);

    run_succeeds([
        "-V", vh_a, "declare", "queue", "--name", q, "--type", "classic",
    ]);

    run_succeeds([
        "-V",
        vh_a,
        "policies",
        "declare",
        "--name",
        policy_name,
        "--pattern",
        "^federation\\.",
        "--apply-to",
        "queues",
        "--priority",
        "98",
        "--definition",
        "{\"federation-upstream-set\": \"all\"}",
    ]);

    await_federation_link_with(upstream_name, 15_000);

    run_succeeds([
        "-V",
        vh_a,
        "policies",
        "declare",
        "--name",
        policy_name,
        "--pattern",
        "^federation\\.",
        "--apply-to",
        "queues",
        "--priority",
        "98",
        "--definition",
        "{\"max-length\": 100}",
    ]);

    await_no_federation_link_with(upstream_name, 15_000);

    delete_vhost(vh_a).ok();
    delete_vhost(vh_b).ok();

    Ok(())
}

#[test]
fn test_federation_link_starts_after_policy_enables_federation() -> Result<(), Box<dyn Error>> {
    let vh_a = "rabbitmqadmin.federation.link_term.2a";
    let vh_b = "rabbitmqadmin.federation.link_term.2b";
    let upstream_name = "up.link_term.policy_enable";
    let policy_name = "pol.link_term.policy_enable";
    let q = "federation.link_term.2";
    let amqp_uri = amqp_endpoint_with_vhost(vh_b);

    delete_vhost(vh_a).ok();
    delete_vhost(vh_b).ok();
    run_succeeds(["declare", "vhost", "--name", vh_a]);
    run_succeeds(["declare", "vhost", "--name", vh_b]);

    run_succeeds([
        "-V",
        vh_a,
        "federation",
        "declare_upstream",
        "--name",
        upstream_name,
        "--uri",
        &amqp_uri,
    ]);

    run_succeeds([
        "-V", vh_a, "declare", "queue", "--name", q, "--type", "classic",
    ]);

    run_succeeds([
        "-V",
        vh_a,
        "policies",
        "declare",
        "--name",
        policy_name,
        "--pattern",
        "^federation\\.",
        "--apply-to",
        "queues",
        "--priority",
        "98",
        "--definition",
        "{\"max-length\": 100}",
    ]);

    await_ms(3000);
    await_no_federation_link_with(upstream_name, 5_000);

    run_succeeds([
        "-V",
        vh_a,
        "policies",
        "declare",
        "--name",
        policy_name,
        "--pattern",
        "^federation\\.",
        "--apply-to",
        "queues",
        "--priority",
        "98",
        "--definition",
        "{\"federation-upstream-set\": \"all\"}",
    ]);

    await_federation_link_with(upstream_name, 15_000);

    delete_vhost(vh_a).ok();
    delete_vhost(vh_b).ok();

    Ok(())
}

#[test]
fn test_federation_link_stops_after_policy_removal() -> Result<(), Box<dyn Error>> {
    let vh_a = "rabbitmqadmin.federation.link_term.3a";
    let vh_b = "rabbitmqadmin.federation.link_term.3b";
    let upstream_name = "up.link_term.policy_remove";
    let policy_name = "pol.link_term.policy_remove";
    let q = "federation.link_term.3";
    let amqp_uri = amqp_endpoint_with_vhost(vh_b);

    delete_vhost(vh_a).ok();
    delete_vhost(vh_b).ok();
    run_succeeds(["declare", "vhost", "--name", vh_a]);
    run_succeeds(["declare", "vhost", "--name", vh_b]);

    run_succeeds([
        "-V",
        vh_a,
        "federation",
        "declare_upstream",
        "--name",
        upstream_name,
        "--uri",
        &amqp_uri,
    ]);

    run_succeeds([
        "-V", vh_a, "declare", "queue", "--name", q, "--type", "classic",
    ]);

    run_succeeds([
        "-V",
        vh_a,
        "policies",
        "declare",
        "--name",
        policy_name,
        "--pattern",
        "^federation\\.",
        "--apply-to",
        "queues",
        "--priority",
        "98",
        "--definition",
        "{\"federation-upstream-set\": \"all\"}",
    ]);

    await_federation_link_with(upstream_name, 15_000);

    run_succeeds(["-V", vh_a, "policies", "delete", "--name", policy_name]);

    await_no_federation_link_with(upstream_name, 15_000);

    delete_vhost(vh_a).ok();
    delete_vhost(vh_b).ok();

    Ok(())
}

#[test]
fn test_federation_link_stops_after_upstream_removal() -> Result<(), Box<dyn Error>> {
    let vh_a = "rabbitmqadmin.federation.link_term.4a";
    let vh_b = "rabbitmqadmin.federation.link_term.4b";
    let upstream_name = "up.link_term.upstream_remove";
    let policy_name = "pol.link_term.upstream_remove";
    let q = "federation.link_term.4";
    let amqp_uri = amqp_endpoint_with_vhost(vh_b);

    delete_vhost(vh_a).ok();
    delete_vhost(vh_b).ok();
    run_succeeds(["declare", "vhost", "--name", vh_a]);
    run_succeeds(["declare", "vhost", "--name", vh_b]);

    run_succeeds([
        "-V",
        vh_a,
        "federation",
        "declare_upstream",
        "--name",
        upstream_name,
        "--uri",
        &amqp_uri,
    ]);

    run_succeeds([
        "-V", vh_a, "declare", "queue", "--name", q, "--type", "classic",
    ]);

    run_succeeds([
        "-V",
        vh_a,
        "policies",
        "declare",
        "--name",
        policy_name,
        "--pattern",
        "^federation\\.",
        "--apply-to",
        "queues",
        "--priority",
        "98",
        "--definition",
        "{\"federation-upstream-set\": \"all\"}",
    ]);

    await_federation_link_with(upstream_name, 15_000);

    run_succeeds([
        "-V",
        vh_a,
        "federation",
        "delete_upstream",
        "--name",
        upstream_name,
    ]);

    await_no_federation_link_with(upstream_name, 15_000);

    delete_vhost(vh_a).ok();
    delete_vhost(vh_b).ok();

    Ok(())
}
