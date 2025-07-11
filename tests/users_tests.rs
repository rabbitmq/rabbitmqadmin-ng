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
use crate::test_helpers::*;

#[test]
fn test_list_users() -> Result<(), Box<dyn std::error::Error>> {
    let username = "test_list_users";
    let password = "pa$$w0rd";
    run_succeeds([
        "users",
        "declare",
        "--name",
        username,
        "--password",
        password,
    ]);

    run_succeeds(["list", "users"]).stdout(predicate::str::contains(username));
    run_succeeds(["delete", "user", "--name", username]);
    run_succeeds(["delete", "user", "--name", username, "--idempotently"]);

    run_succeeds(["list", "users"]).stdout(predicate::str::contains(username).not());

    Ok(())
}

#[test]
fn test_users_list() -> Result<(), Box<dyn std::error::Error>> {
    let username = "test_users_list.2";
    let password = "pa$$w0rd";
    run_succeeds([
        "users",
        "declare",
        "--name",
        username,
        "--password",
        password,
    ]);

    run_succeeds(["users", "list"]).stdout(predicate::str::contains(username));
    run_succeeds(["users", "delete", "--name", username]);
    run_succeeds(["users", "delete", "--name", username, "--idempotently"]);

    run_succeeds(["users", "list"]).stdout(predicate::str::contains(username).not());

    Ok(())
}

#[test]
fn test_list_users_with_table_styles() -> Result<(), Box<dyn std::error::Error>> {
    let username = "test_list_users_with_table_styles";
    let password = "pa$$w0rd";
    run_succeeds([
        "declare",
        "user",
        "--name",
        username,
        "--password",
        password,
    ]);

    run_succeeds(["--table-style", "markdown", "list", "users"])
        .stdout(predicate::str::contains(username));
    run_succeeds(["delete", "user", "--name", username]);
    run_succeeds(["delete", "user", "--name", username, "--idempotently"]);

    run_succeeds(["--table-style", "borderless", "list", "users"])
        .stdout(predicate::str::contains(username).not());

    Ok(())
}

#[test]
fn test_create_user_using_sha256_for_hashing() -> Result<(), Box<dyn std::error::Error>> {
    let username = "test_create_user_using_sha256_for_hashing.1";
    let password = "pa$$w0rd_9w798f__sd8f7";

    run_succeeds(["users", "delete", "--name", username, "--idempotently"]);

    run_succeeds([
        "users",
        "declare",
        "--name",
        username,
        "--password",
        password,
        "--hashing-algorithm",
        "sha256",
        "--tags",
        "administrator",
    ]);

    run_succeeds(["--username", username, "--password", password, "users", "list"]).stdout(predicate::str::contains(username));
    run_succeeds(["users", "delete", "--name", username]);

    run_succeeds(["list", "users"]).stdout(predicate::str::contains(username).not());

    Ok(())
}

#[test]
fn test_create_user_using_sha512_for_hashing() -> Result<(), Box<dyn std::error::Error>> {
    let username = "test_create_user_using_sha512_for_hashing.1";
    let password = "pa$$w0rd///8*9";

    run_succeeds(["users", "delete", "--name", username, "--idempotently"]);

    run_succeeds([
        "users",
        "declare",
        "--name",
        username,
        "--password",
        password,
        "--hashing-algorithm",
        "sha512",
        "--tags",
        "administrator",
    ]);
    // unless the node is also configured to use SHA-512, we cannot try this
    // password the same way we do in the SHA-256 version, by passing in --username and --password
    run_succeeds(["users", "delete", "--name", username]);

    run_succeeds(["list", "users"]).stdout(predicate::str::contains(username).not());

    Ok(())
}