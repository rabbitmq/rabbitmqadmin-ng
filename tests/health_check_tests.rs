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
use predicates::prelude::*;

mod test_helpers;
use test_helpers::{run_fails, run_succeeds};

#[test]
fn test_health_check_local_alarms() -> Result<(), Box<dyn std::error::Error>> {
    run_succeeds(["health_check", "local_alarms"]).stdout(predicate::str::contains("passed"));

    Ok(())
}

#[test]
fn test_health_check_cluster_wide_alarms() -> Result<(), Box<dyn std::error::Error>> {
    run_succeeds(["health_check", "cluster_wide_alarms"])
        .stdout(predicate::str::contains("passed"));

    Ok(())
}

#[test]
fn test_health_check_port_listener_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    run_succeeds(["health_check", "port_listener", "--port", "15672"])
        .stdout(predicate::str::contains("passed"));

    Ok(())
}

#[test]
fn test_health_check_port_listener_fails() -> Result<(), Box<dyn std::error::Error>> {
    run_fails(["health_check", "port_listener", "--port", "15679"])
        .stdout(predicate::str::contains("failed"));

    Ok(())
}

#[test]
fn test_health_check_protocol_listener_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    run_succeeds(["health_check", "protocol_listener", "--protocol", "amqp"])
        .stdout(predicate::str::contains("passed"));

    Ok(())
}

#[test]
fn test_health_check_protocol_listener_fails() -> Result<(), Box<dyn std::error::Error>> {
    run_fails([
        "health_check",
        "protocol_listener",
        "--protocol",
        "https/prometheus",
    ])
    .stdout(predicate::str::contains("failed"));
    run_fails([
        "health_check",
        "protocol_listener",
        "--protocol",
        "unknown/proto",
    ])
    .stdout(predicate::str::contains("failed"));

    Ok(())
}
