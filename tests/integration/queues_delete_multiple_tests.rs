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

use crate::test_helpers::*;
use predicates::prelude::*;
use rabbitmq_http_client::blocking_api::Client as GenericAPIClient;
use std::error::Error;

type APIClient<'a> = GenericAPIClient<&'a str, &'a str, &'a str>;

// Each test owns a distinct virtual host so tests can run in parallel without
// clobbering each other's queue declarations.
fn test_vhost(suffix: &str) -> String {
    format!("rabbitmqadmin.test-queues-delete-multiple-{suffix}")
}

fn declare_vhost(vhost: &str) {
    delete_vhost(vhost).ok();
    run_succeeds(["vhosts", "declare", "--name", vhost]);
}

fn declare_classic_queue(vhost: &str, name: &str) {
    run_succeeds([
        "--vhost", vhost, "queues", "declare", "--name", name, "--type", "classic",
    ]);
}

fn queue_names_in(client: &APIClient<'_>, vhost: &str) -> Vec<String> {
    client
        .list_queues_in(vhost)
        .unwrap_or_default()
        .into_iter()
        .map(|q| q.name)
        .collect()
}

#[test]
fn test_queues_delete_multiple_basic() -> Result<(), Box<dyn Error>> {
    let vhost = test_vhost("basic");
    declare_vhost(&vhost);

    for i in 1..=5 {
        declare_classic_queue(&vhost, &format!("leftover-{i}"));
    }

    let client = api_client();
    assert_eq!(queue_names_in(&client, &vhost).len(), 5);

    run_succeeds([
        "--vhost",
        &vhost,
        "queues",
        "delete_multiple",
        "--name-pattern",
        "^leftover-.*",
        "--approve",
        "--idempotently",
    ]);

    assert_eq!(queue_names_in(&client, &vhost).len(), 0);

    delete_vhost(&vhost).ok();
    Ok(())
}

#[test]
fn test_queues_delete_multiple_dry_run() -> Result<(), Box<dyn Error>> {
    let vhost = test_vhost("dry-run");
    declare_vhost(&vhost);

    for i in 1..=3 {
        declare_classic_queue(&vhost, &format!("leftover-{i}"));
    }

    let client = api_client();
    assert_eq!(queue_names_in(&client, &vhost).len(), 3);

    run_succeeds([
        "--vhost",
        &vhost,
        "queues",
        "delete_multiple",
        "--name-pattern",
        "^leftover-.*",
        "--dry-run",
    ])
    .stdout(output_includes("leftover-1"))
    .stdout(output_includes("leftover-2"))
    .stdout(output_includes("leftover-3"));

    assert_eq!(queue_names_in(&client, &vhost).len(), 3);

    delete_vhost(&vhost).ok();
    Ok(())
}

#[test]
fn test_queues_delete_multiple_non_interactive() -> Result<(), Box<dyn Error>> {
    let vhost = test_vhost("non-interactive");
    declare_vhost(&vhost);

    for i in 1..=2 {
        declare_classic_queue(&vhost, &format!("leftover-{i}"));
    }

    let client = api_client();
    assert_eq!(queue_names_in(&client, &vhost).len(), 2);

    run_succeeds([
        "--non-interactive",
        "--vhost",
        &vhost,
        "queues",
        "delete_multiple",
        "--name-pattern",
        "^leftover-.*",
        "--idempotently",
    ]);

    assert_eq!(queue_names_in(&client, &vhost).len(), 0);

    delete_vhost(&vhost).ok();
    Ok(())
}

#[test]
fn test_queues_delete_multiple_with_invalid_regex() -> Result<(), Box<dyn Error>> {
    let vhost = test_vhost("invalid-regex");
    declare_vhost(&vhost);

    declare_classic_queue(&vhost, "leftover-1");

    run_fails([
        "--vhost",
        &vhost,
        "queues",
        "delete_multiple",
        "--name-pattern",
        "[invalid",
        "--approve",
    ]);

    let client = api_client();
    assert_eq!(
        queue_names_in(&client, &vhost),
        vec!["leftover-1".to_string()]
    );

    delete_vhost(&vhost).ok();
    Ok(())
}

#[test]
fn test_queues_delete_multiple_requires_approve_in_interactive_mode() -> Result<(), Box<dyn Error>>
{
    let vhost = test_vhost("requires-approve");
    declare_vhost(&vhost);

    declare_classic_queue(&vhost, "leftover-1");

    run_fails([
        "--vhost",
        &vhost,
        "queues",
        "delete_multiple",
        "--name-pattern",
        "^leftover-.*",
    ]);

    let client = api_client();
    assert_eq!(
        queue_names_in(&client, &vhost),
        vec!["leftover-1".to_string()]
    );

    delete_vhost(&vhost).ok();
    Ok(())
}

// Pre-deletes one matching queue and runs the bulk command with --idempotently
// to verify the loop tolerates per-item 404s and keeps deleting the rest.
#[test]
fn test_queues_delete_multiple_continues_on_individual_failures() -> Result<(), Box<dyn Error>> {
    let vhost = test_vhost("continues");
    declare_vhost(&vhost);

    for i in 1..=3 {
        declare_classic_queue(&vhost, &format!("leftover-{i}"));
    }

    let client = api_client();
    assert_eq!(queue_names_in(&client, &vhost).len(), 3);

    run_succeeds([
        "--vhost",
        &vhost,
        "queues",
        "delete",
        "--name",
        "leftover-2",
    ]);
    assert_eq!(queue_names_in(&client, &vhost).len(), 2);

    run_succeeds([
        "--vhost",
        &vhost,
        "queues",
        "delete_multiple",
        "--name-pattern",
        "^leftover-.*",
        "--approve",
        "--idempotently",
    ]);

    assert_eq!(queue_names_in(&client, &vhost).len(), 0);

    delete_vhost(&vhost).ok();
    Ok(())
}

#[test]
fn test_queues_delete_multiple_no_matches() -> Result<(), Box<dyn Error>> {
    let vhost = test_vhost("no-matches");
    declare_vhost(&vhost);

    declare_classic_queue(&vhost, "keep-me");

    let client = api_client();
    let before = queue_names_in(&client, &vhost);

    run_succeeds([
        "--vhost",
        &vhost,
        "queues",
        "delete_multiple",
        "--name-pattern",
        "^this-prefix-matches-nothing-.*$",
        "--approve",
    ]);

    let after = queue_names_in(&client, &vhost);
    assert_eq!(before, after);

    delete_vhost(&vhost).ok();
    Ok(())
}

// Same-named queues live in two vhosts: the bulk delete must only touch the
// vhost named by --vhost, not the other one.
#[test]
fn test_queues_delete_multiple_scoped_to_vhost() -> Result<(), Box<dyn Error>> {
    let target_vhost = test_vhost("scoped-target");
    let bystander_vhost = test_vhost("scoped-bystander");

    declare_vhost(&target_vhost);
    declare_vhost(&bystander_vhost);

    for i in 1..=2 {
        let name = format!("leftover-{i}");
        declare_classic_queue(&target_vhost, &name);
        declare_classic_queue(&bystander_vhost, &name);
    }

    let client = api_client();
    assert_eq!(queue_names_in(&client, &target_vhost).len(), 2);
    assert_eq!(queue_names_in(&client, &bystander_vhost).len(), 2);

    run_succeeds([
        "--vhost",
        &target_vhost,
        "queues",
        "delete_multiple",
        "--name-pattern",
        "^leftover-.*",
        "--approve",
        "--idempotently",
    ]);

    assert_eq!(queue_names_in(&client, &target_vhost).len(), 0);
    assert_eq!(queue_names_in(&client, &bystander_vhost).len(), 2);

    delete_vhost(&target_vhost).ok();
    delete_vhost(&bystander_vhost).ok();
    Ok(())
}

#[test]
fn test_queues_delete_multiple_quiet_produces_no_output() -> Result<(), Box<dyn Error>> {
    let vhost = test_vhost("quiet");
    declare_vhost(&vhost);

    for i in 1..=3 {
        declare_classic_queue(&vhost, &format!("leftover-{i}"));
    }

    run_succeeds([
        "--quiet",
        "--non-interactive",
        "--vhost",
        &vhost,
        "queues",
        "delete_multiple",
        "--name-pattern",
        "^leftover-.*",
        "--idempotently",
    ])
    .stdout(predicate::str::is_empty());

    let client = api_client();
    assert_eq!(queue_names_in(&client, &vhost).len(), 0);

    delete_vhost(&vhost).ok();
    Ok(())
}

#[test]
fn test_queues_delete_multiple_partial_match_only() -> Result<(), Box<dyn Error>> {
    let vhost = test_vhost("partial");
    declare_vhost(&vhost);

    declare_classic_queue(&vhost, "leftover-target");
    declare_classic_queue(&vhost, "leftover-keep-me");

    run_succeeds([
        "--vhost",
        &vhost,
        "queues",
        "delete_multiple",
        "--name-pattern",
        "^leftover-target$",
        "--approve",
    ]);

    let client = api_client();
    assert_eq!(
        queue_names_in(&client, &vhost),
        vec!["leftover-keep-me".to_string()]
    );

    delete_vhost(&vhost).ok();
    Ok(())
}

#[test]
fn test_queues_delete_multiple_dry_run_with_no_matches_is_silent_success()
-> Result<(), Box<dyn Error>> {
    let vhost = test_vhost("dry-run-empty");
    declare_vhost(&vhost);

    declare_classic_queue(&vhost, "keep-me");

    run_succeeds([
        "--vhost",
        &vhost,
        "queues",
        "delete_multiple",
        "--name-pattern",
        "^nothing-matches-this-.*$",
        "--dry-run",
    ]);

    let client = api_client();
    assert_eq!(queue_names_in(&client, &vhost), vec!["keep-me".to_string()]);

    delete_vhost(&vhost).ok();
    Ok(())
}

#[test]
fn test_queues_delete_multiple_dry_run_does_not_require_approve() -> Result<(), Box<dyn Error>> {
    let vhost = test_vhost("dry-run-no-approve");
    declare_vhost(&vhost);

    declare_classic_queue(&vhost, "leftover-1");

    run_succeeds([
        "--vhost",
        &vhost,
        "queues",
        "delete_multiple",
        "--name-pattern",
        "^leftover-.*",
        "--dry-run",
    ]);

    let client = api_client();
    assert_eq!(
        queue_names_in(&client, &vhost),
        vec!["leftover-1".to_string()]
    );

    delete_vhost(&vhost).ok();
    Ok(())
}

#[test]
fn test_queues_delete_multiple_help_includes_danger_zone() -> Result<(), Box<dyn Error>> {
    run_succeeds(["queues", "delete_multiple", "--help"])
        .stdout(output_includes("DANGER ZONE"))
        .stdout(output_includes("--name-pattern"))
        .stdout(output_includes("--approve"))
        .stdout(output_includes("--dry-run"))
        .stdout(output_includes("--idempotently"));
    Ok(())
}
