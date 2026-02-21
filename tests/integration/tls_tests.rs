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

//! TLS integration tests.
//!
//! These tests are ignored by default and require a RabbitMQ node with TLS-enabled
//! HTTP API running on localhost:15671.
//!
//! To run these tests:
//!
//! 1. Generate certificates using [tls-gen](https://github.com/rabbitmq/tls-gen):
//!    cd /path/to/tls-gen/basic && make CN=localhost && make alias-leaf-artifacts
//!
//! 2. [Configure](https://www.rabbitmq.com/docs/management#single-listener-https) RabbitMQ management plugin to use TLS
//!
//! 3. Export the `TLS_CERTS_DIR` environment variable:
//!    export TLS_CERTS_DIR=/path/to/tls-gen/basic/result
//!
//! 4. Run bin/ci/before_build_tls.sh instead of the standard bin/ci/before_build.sh to set the node up
//!
//! 5. Run the tests in this module only:
//!    cargo nextest run -E 'test(/^tls_tests::/)' --run-ignored=only

use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

use assert_cmd::assert::Assert;
use assert_cmd::prelude::*;
use predicates::prelude::*;

const TLS_ENDPOINT: &str = "https://localhost:15671/api";
const TLS_PORT: &str = "15671";
const USERNAME: &str = "guest";
const PASSWORD: &str = "guest";

fn tls_certs_dir() -> PathBuf {
    env::var("TLS_CERTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(manifest_dir).join("tests/tls/certs")
        })
}

fn ca_cert_path() -> String {
    tls_certs_dir()
        .join("ca_certificate.pem")
        .to_string_lossy()
        .to_string()
}

fn client_cert_path() -> String {
    tls_certs_dir()
        .join("client_certificate.pem")
        .to_string_lossy()
        .to_string()
}

fn client_key_path() -> String {
    tls_certs_dir()
        .join("client_key.pem")
        .to_string_lossy()
        .to_string()
}

fn run_tls_succeeds<I, S>(args: I) -> Assert
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("rabbitmqadmin"));
    cmd.args([
        "--base-uri",
        TLS_ENDPOINT,
        "--username",
        USERNAME,
        "--password",
        PASSWORD,
        "--use-tls",
        "--tls-ca-cert-file",
        &ca_cert_path(),
    ]);
    cmd.args(args).assert().success()
}

fn run_tls_with_client_cert_succeeds<I, S>(args: I) -> Assert
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("rabbitmqadmin"));
    cmd.args([
        "--base-uri",
        TLS_ENDPOINT,
        "--username",
        USERNAME,
        "--password",
        PASSWORD,
        "--use-tls",
        "--tls-ca-cert-file",
        &ca_cert_path(),
        "--tls-cert-file",
        &client_cert_path(),
        "--tls-key-file",
        &client_key_path(),
    ]);
    cmd.args(args).assert().success()
}

fn run_tls_insecure_succeeds<I, S>(args: I) -> Assert
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("rabbitmqadmin"));
    cmd.args([
        "--host",
        "localhost",
        "--port",
        TLS_PORT,
        "--username",
        USERNAME,
        "--password",
        PASSWORD,
        "--use-tls",
        "--insecure",
    ]);
    cmd.args(args).assert().success()
}

#[allow(dead_code)]
fn run_tls_fails<I, S>(args: I) -> Assert
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("rabbitmqadmin"));
    cmd.args([
        "--base-uri",
        TLS_ENDPOINT,
        "--username",
        USERNAME,
        "--password",
        PASSWORD,
        "--use-tls",
        "--tls-ca-cert-file",
        &ca_cert_path(),
    ]);
    cmd.args(args).assert().failure()
}

fn output_includes(content: &str) -> predicates::str::ContainsPredicate {
    predicate::str::contains(content)
}

#[test]
#[ignore]
fn test_tls_show_overview() -> Result<(), Box<dyn Error>> {
    run_tls_succeeds(["show", "overview"]).stdout(
        output_includes("RabbitMQ version")
            .and(output_includes("Product name"))
            .and(output_includes("Product version")),
    );

    Ok(())
}

#[test]
#[ignore]
fn test_tls_show_overview_insecure() -> Result<(), Box<dyn Error>> {
    run_tls_insecure_succeeds(["show", "overview"]).stdout(
        output_includes("RabbitMQ version")
            .and(output_includes("Product name"))
            .and(output_includes("Product version")),
    );

    Ok(())
}

#[test]
#[ignore]
fn test_tls_list_nodes() -> Result<(), Box<dyn Error>> {
    run_tls_succeeds(["list", "nodes"]).stdout(output_includes("rabbit@"));

    Ok(())
}

#[test]
#[ignore]
fn test_tls_list_vhosts() -> Result<(), Box<dyn Error>> {
    run_tls_succeeds(["list", "vhosts"]).stdout(output_includes("/"));

    Ok(())
}

#[test]
#[ignore]
fn test_tls_list_users() -> Result<(), Box<dyn Error>> {
    run_tls_succeeds(["list", "users"]).stdout(output_includes("guest"));

    Ok(())
}

#[test]
#[ignore]
fn test_tls_health_check_local_alarms() -> Result<(), Box<dyn Error>> {
    run_tls_succeeds(["health_check", "local_alarms"]).stdout(output_includes("passed"));

    Ok(())
}

#[test]
#[ignore]
fn test_tls_health_check_cluster_wide_alarms() -> Result<(), Box<dyn Error>> {
    run_tls_succeeds(["health_check", "cluster_wide_alarms"]).stdout(output_includes("passed"));

    Ok(())
}

#[test]
#[ignore]
fn test_tls_show_endpoint() -> Result<(), Box<dyn Error>> {
    run_tls_succeeds(["show", "endpoint"]).stdout(output_includes("https://localhost:15671"));

    Ok(())
}

#[test]
#[ignore]
fn test_tls_with_client_certificate_show_overview() -> Result<(), Box<dyn Error>> {
    run_tls_with_client_cert_succeeds(["show", "overview"]).stdout(
        output_includes("RabbitMQ version")
            .and(output_includes("Product name"))
            .and(output_includes("Product version")),
    );

    Ok(())
}

#[test]
#[ignore]
fn test_tls_with_client_certificate_list_nodes() -> Result<(), Box<dyn Error>> {
    run_tls_with_client_cert_succeeds(["list", "nodes"]).stdout(output_includes("rabbit@"));

    Ok(())
}
