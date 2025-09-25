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
#![allow(dead_code)]

use std::env;
use std::ffi::OsStr;
use std::time::Duration;

use assert_cmd::assert::Assert;
use assert_cmd::prelude::*;
use std::process::Command;

use rabbitmq_http_client::blocking_api::Client as GenericAPIClient;

type APIClient<'a> = GenericAPIClient<&'a str, &'a str, &'a str>;

type CommandRunResult = Result<(), Box<dyn std::error::Error>>;

pub const ENDPOINT: &str = "http://localhost:15672/api";
pub const USERNAME: &str = "guest";
pub const PASSWORD: &str = "guest";

pub const AMQP_ENDPOINT: &str = "amqp://localhost:5672";

pub fn endpoint() -> String {
    ENDPOINT.to_owned()
}

pub fn api_client() -> APIClient<'static> {
    APIClient::new(ENDPOINT, USERNAME, PASSWORD)
}

pub fn amqp_endpoint() -> String {
    AMQP_ENDPOINT.to_owned()
}

pub fn amqp_endpoint_with_vhost(name: &str) -> String {
    format!("{0}/{1}", AMQP_ENDPOINT, name).to_owned()
}

pub fn await_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

pub fn await_metric_emission(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

pub fn await_queue_metric_emission() {
    let delay = env::var("TEST_STATS_DELAY").unwrap_or("500".to_owned());
    await_metric_emission(delay.parse::<u64>().unwrap());
}

pub fn run_succeeds<I, S>(args: I) -> Assert
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::cargo_bin("rabbitmqadmin").unwrap();
    cmd.args(args).assert().success()
}

pub fn run_fails<I, S>(args: I) -> Assert
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::cargo_bin("rabbitmqadmin").unwrap();
    cmd.args(args).assert().failure()
}

pub fn create_vhost(vhost: &str) -> CommandRunResult {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["vhosts", "declare", "--name", vhost]);
    cmd.assert().success();
    Ok(())
}

pub fn delete_vhost(vhost: &str) -> CommandRunResult {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["vhosts", "delete", "--name", vhost, "--idempotently"]);
    cmd.assert().success();
    Ok(())
}

pub fn delete_user(username: &str) -> CommandRunResult {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "user", "--name", username, "--idempotently"]);
    cmd.assert().success();
    Ok(())
}

pub fn delete_all_test_vhosts() -> CommandRunResult {
    let client = api_client();
    match client.list_vhosts() {
        Ok(vhosts) => {
            for vhost in vhosts {
                if vhost.name.starts_with("rabbitmqadmin.") {
                    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
                    cmd.args(["vhosts", "delete", "--name", &vhost.name, "--idempotently"]);
                    let _ = cmd.assert().success();
                }
            }
        }
        Err(_) => {
            // If we can't list vhosts, continue anyway
        }
    }
    Ok(())
}
