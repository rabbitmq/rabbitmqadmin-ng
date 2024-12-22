// Copyright (C) 2023-2024 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_health_check_local_alarms() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.args(["health_check", "local_alarms"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("passed"));

    Ok(())
}

#[test]
fn test_health_check_cluster_wide_alarms() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.args(["health_check", "cluster_wide_alarms"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("passed"));

    Ok(())
}
