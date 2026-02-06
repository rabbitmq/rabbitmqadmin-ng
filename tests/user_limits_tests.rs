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

use predicates::prelude::*;
use std::error::Error;

mod test_helpers;
use crate::test_helpers::*;

#[test]
fn test_user_limits() -> Result<(), Box<dyn Error>> {
    let limit_name = "max-connections";
    let username = "user_limits_test1";
    let password = "pa$$w0rd";
    run_succeeds([
        "declare",
        "user",
        "--name",
        username,
        "--password",
        password,
    ]);

    run_succeeds([
        "user_limits",
        "declare",
        "--username",
        username,
        "--name",
        limit_name,
        "--value",
        "1234",
    ]);

    run_succeeds(["user_limits", "list"])
        .stdout(output_includes(username).and(output_includes("1234")));

    run_succeeds([
        "user_limits",
        "delete",
        "--username",
        username,
        "--name",
        limit_name,
    ]);

    run_succeeds(["user_limits", "list"]).stdout(output_includes(username).not());

    // Deleting a non-existent limit is idempotent
    run_succeeds([
        "user_limits",
        "delete",
        "--username",
        username,
        "--name",
        limit_name,
    ]);

    run_succeeds(["delete", "user", "--name", username]);

    Ok(())
}

#[test]
fn test_user_limits_via_old_style_commands() -> Result<(), Box<dyn Error>> {
    let limit_name = "max-connections";
    let username = "user_limits_old_style";
    let password = "pa$$w0rd";
    run_succeeds([
        "declare",
        "user",
        "--name",
        username,
        "--password",
        password,
    ]);

    run_succeeds([
        "declare",
        "user_limit",
        "--username",
        username,
        "--name",
        limit_name,
        "--value",
        "100",
    ]);

    run_succeeds(["list", "user_limits", "--username", username])
        .stdout(output_includes(limit_name).and(output_includes("100")));

    run_succeeds([
        "delete",
        "user_limit",
        "--username",
        username,
        "--name",
        limit_name,
    ]);

    run_succeeds(["list", "user_limits"]).stdout(output_includes(username).not());
    run_succeeds(["delete", "user", "--name", username]);

    Ok(())
}

#[test]
fn test_user_limits_with_deprecated_user_flag() -> Result<(), Box<dyn Error>> {
    let limit_name = "max-connections";
    let username = "user_limits_compat";
    let password = "pa$$w0rd";
    run_succeeds([
        "declare",
        "user",
        "--name",
        username,
        "--password",
        password,
    ]);

    // --user is a backward-compatible alias for --username
    run_succeeds([
        "user_limits",
        "declare",
        "--user",
        username,
        "--name",
        limit_name,
        "--value",
        "5678",
    ]);

    run_succeeds(["user_limits", "list"])
        .stdout(output_includes(username).and(output_includes("5678")));

    run_succeeds([
        "user_limits",
        "delete",
        "--user",
        username,
        "--name",
        limit_name,
    ]);

    run_succeeds(["user_limits", "list"]).stdout(output_includes(username).not());
    run_succeeds(["delete", "user", "--name", username]);

    Ok(())
}

#[test]
fn test_user_limits_with_deprecated_user_flag_via_old_style_commands() -> Result<(), Box<dyn Error>>
{
    let limit_name = "max-connections";
    let username = "user_limits_old_compat";
    let password = "pa$$w0rd";
    run_succeeds([
        "declare",
        "user",
        "--name",
        username,
        "--password",
        password,
    ]);

    // --user is a backward-compatible alias for --username
    run_succeeds([
        "declare",
        "user_limit",
        "--user",
        username,
        "--name",
        limit_name,
        "--value",
        "200",
    ]);

    run_succeeds(["list", "user_limits", "--user", username])
        .stdout(output_includes(limit_name).and(output_includes("200")));

    run_succeeds([
        "delete",
        "user_limit",
        "--user",
        username,
        "--name",
        limit_name,
    ]);

    run_succeeds(["list", "user_limits"]).stdout(output_includes(username).not());
    run_succeeds(["delete", "user", "--name", username]);

    Ok(())
}

#[test]
fn test_user_limits_via_users_command_group() -> Result<(), Box<dyn Error>> {
    let limit_name = "max-connections";
    let username = "user_limits_via_users";
    let password = "pa$$w0rd";
    run_succeeds([
        "declare",
        "user",
        "--name",
        username,
        "--password",
        password,
    ]);

    run_succeeds([
        "user_limits",
        "declare",
        "--username",
        username,
        "--name",
        limit_name,
        "--value",
        "300",
    ]);

    run_succeeds(["users", "limits", "--username", username])
        .stdout(output_includes(limit_name).and(output_includes("300")));

    run_succeeds([
        "user_limits",
        "delete",
        "--username",
        username,
        "--name",
        limit_name,
    ]);

    run_succeeds(["users", "limits"]).stdout(output_includes(username).not());
    run_succeeds(["delete", "user", "--name", username]);

    Ok(())
}

#[test]
fn test_user_limits_list_filtering_by_username() -> Result<(), Box<dyn Error>> {
    let username_a = "user_limits_filter_a";
    let username_b = "user_limits_filter_b";
    let password = "pa$$w0rd";

    run_succeeds([
        "declare",
        "user",
        "--name",
        username_a,
        "--password",
        password,
    ]);
    run_succeeds([
        "declare",
        "user",
        "--name",
        username_b,
        "--password",
        password,
    ]);

    run_succeeds([
        "user_limits",
        "declare",
        "--username",
        username_a,
        "--name",
        "max-connections",
        "--value",
        "111",
    ]);
    run_succeeds([
        "user_limits",
        "declare",
        "--username",
        username_b,
        "--name",
        "max-connections",
        "--value",
        "222",
    ]);

    // Unfiltered list includes both users
    run_succeeds(["user_limits", "list"])
        .stdout(output_includes(username_a).and(output_includes(username_b)));

    // Filtered list includes only the requested user
    run_succeeds(["user_limits", "list", "--username", username_a])
        .stdout(output_includes("111").and(output_includes(username_b).not()));

    run_succeeds([
        "user_limits",
        "delete",
        "--username",
        username_a,
        "--name",
        "max-connections",
    ]);
    run_succeeds([
        "user_limits",
        "delete",
        "--username",
        username_b,
        "--name",
        "max-connections",
    ]);
    run_succeeds(["delete", "user", "--name", username_a]);
    run_succeeds(["delete", "user", "--name", username_b]);

    Ok(())
}

#[test]
fn test_user_limits_via_users_command_group_with_deprecated_user_flag() -> Result<(), Box<dyn Error>>
{
    let limit_name = "max-connections";
    let username = "user_limits_users_compat";
    let password = "pa$$w0rd";
    run_succeeds([
        "declare",
        "user",
        "--name",
        username,
        "--password",
        password,
    ]);

    run_succeeds([
        "user_limits",
        "declare",
        "--username",
        username,
        "--name",
        limit_name,
        "--value",
        "400",
    ]);

    // --user is a backward-compatible alias for --username
    run_succeeds(["users", "limits", "--user", username])
        .stdout(output_includes(limit_name).and(output_includes("400")));

    run_succeeds([
        "user_limits",
        "delete",
        "--username",
        username,
        "--name",
        limit_name,
    ]);
    run_succeeds(["delete", "user", "--name", username]);

    Ok(())
}

#[test]
fn test_user_limits_with_separate_auth_and_target_usernames() -> Result<(), Box<dyn Error>> {
    let target = "user_limits_separate_auth";
    let password = "pa$$w0rd";
    run_succeeds(["declare", "user", "--name", target, "--password", password]);

    // Global --username is for API auth, per-command --username is the limit target
    run_succeeds([
        "--username",
        "guest",
        "--password",
        "guest",
        "user_limits",
        "declare",
        "--username",
        target,
        "--name",
        "max-connections",
        "--value",
        "500",
    ]);

    run_succeeds([
        "--username",
        "guest",
        "--password",
        "guest",
        "user_limits",
        "list",
        "--username",
        target,
    ])
    .stdout(output_includes(target).and(output_includes("500")));

    run_succeeds([
        "--username",
        "guest",
        "--password",
        "guest",
        "user_limits",
        "delete",
        "--username",
        target,
        "--name",
        "max-connections",
    ]);

    run_succeeds(["user_limits", "list"]).stdout(output_includes(target).not());
    run_succeeds(["delete", "user", "--name", target]);

    Ok(())
}

#[test]
fn test_user_limits_with_invalid_value() {
    run_fails([
        "user_limits",
        "declare",
        "--username",
        "guest",
        "--name",
        "max-connections",
        "--value",
        "not-a-number",
    ])
    .stderr(output_includes("not a valid integer"));
}
