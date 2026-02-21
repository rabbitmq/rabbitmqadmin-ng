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
use std::error::Error;

#[test]
fn test_list_permissions() -> Result<(), Box<dyn Error>> {
    let username = "user_with_permissions";
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
        "permissions",
        "declare",
        "--username",
        username,
        "--configure",
        "foo",
        "--read",
        "bar",
        "--write",
        "baz",
    ]);

    run_succeeds(["permissions", "list"]).stdout(
        output_includes("foo")
            .and(output_includes("bar"))
            .and(output_includes("baz")),
    );

    run_succeeds(["permissions", "delete", "--username", username]);
    run_succeeds(["permissions", "list"]).stdout(output_includes(username).not());
    run_succeeds(["delete", "user", "--name", username]);

    Ok(())
}

#[test]
fn test_permissions_via_old_style_commands() -> Result<(), Box<dyn Error>> {
    let username = "user_perms_old_style";
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
        "permissions",
        "--username",
        username,
        "--configure",
        ".*",
        "--read",
        ".*",
        "--write",
        ".*",
    ]);

    run_succeeds(["list", "permissions"]).stdout(output_includes(username));

    run_succeeds(["delete", "permissions", "--username", username]);
    run_succeeds(["list", "permissions"]).stdout(output_includes(username).not());
    run_succeeds(["delete", "user", "--name", username]);

    Ok(())
}

#[test]
fn test_permissions_with_deprecated_user_flag() -> Result<(), Box<dyn Error>> {
    let username = "user_with_permissions_compat";
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
        "permissions",
        "declare",
        "--user",
        username,
        "--configure",
        ".*",
        "--read",
        ".*",
        "--write",
        ".*",
    ]);

    run_succeeds(["permissions", "list"]).stdout(output_includes(username));

    run_succeeds(["permissions", "delete", "--user", username]);
    run_succeeds(["permissions", "list"]).stdout(output_includes(username).not());
    run_succeeds(["delete", "user", "--name", username]);

    Ok(())
}

#[test]
fn test_permissions_with_deprecated_user_flag_via_old_style_commands() -> Result<(), Box<dyn Error>>
{
    let username = "user_perms_old_compat";
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
        "permissions",
        "--user",
        username,
        "--configure",
        ".*",
        "--read",
        ".*",
        "--write",
        ".*",
    ]);

    run_succeeds(["list", "permissions"]).stdout(output_includes(username));

    run_succeeds(["delete", "permissions", "--user", username]);
    run_succeeds(["list", "permissions"]).stdout(output_includes(username).not());
    run_succeeds(["delete", "user", "--name", username]);

    Ok(())
}

#[test]
fn test_permissions_idempotent_deletion() -> Result<(), Box<dyn Error>> {
    let username = "user_perms_idempotent";
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
        "permissions",
        "declare",
        "--username",
        username,
        "--configure",
        ".*",
        "--read",
        ".*",
        "--write",
        ".*",
    ]);

    run_succeeds(["permissions", "delete", "--username", username]);
    // Second deletion with --idempotently should succeed
    run_succeeds([
        "permissions",
        "delete",
        "--username",
        username,
        "--idempotently",
    ]);
    // Without --idempotently, deleting non-existent permissions should fail
    run_fails(["permissions", "delete", "--username", username]);

    run_succeeds(["delete", "user", "--name", username]);

    Ok(())
}

#[test]
fn test_permissions_on_a_non_default_vhost() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.permissions.vhost_test";
    let username = "user_perms_vhost";
    let password = "pa$$w0rd";

    run_succeeds(["declare", "vhost", "--name", vh]);
    run_succeeds([
        "declare",
        "user",
        "--name",
        username,
        "--password",
        password,
    ]);

    run_succeeds([
        "--vhost",
        vh,
        "permissions",
        "declare",
        "--username",
        username,
        "--configure",
        "^my_",
        "--read",
        ".*",
        "--write",
        "^my_",
    ]);

    run_succeeds(["permissions", "list"]).stdout(output_includes(username));

    run_succeeds([
        "--vhost",
        vh,
        "permissions",
        "delete",
        "--username",
        username,
    ]);
    run_succeeds(["permissions", "list"]).stdout(output_includes(username).not());

    run_succeeds(["delete", "user", "--name", username]);
    crate::test_helpers::delete_vhost(vh)
}

#[test]
fn test_permissions_on_a_non_default_vhost_via_old_style_commands() -> Result<(), Box<dyn Error>> {
    let vh = "rabbitmqadmin.permissions.vhost_old_style";
    let username = "user_perms_vhost_old";
    let password = "pa$$w0rd";

    run_succeeds(["declare", "vhost", "--name", vh]);
    run_succeeds([
        "declare",
        "user",
        "--name",
        username,
        "--password",
        password,
    ]);

    run_succeeds([
        "-V",
        vh,
        "declare",
        "permissions",
        "--username",
        username,
        "--configure",
        ".*",
        "--read",
        ".*",
        "--write",
        ".*",
    ]);

    run_succeeds(["list", "permissions"]).stdout(output_includes(username));

    run_succeeds(["-V", vh, "delete", "permissions", "--username", username]);
    run_succeeds(["list", "permissions"]).stdout(output_includes(username).not());

    run_succeeds(["delete", "user", "--name", username]);
    crate::test_helpers::delete_vhost(vh)
}

#[test]
fn test_permissions_via_users_command_group() -> Result<(), Box<dyn Error>> {
    let username = "user_perms_via_users";
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
        "permissions",
        "declare",
        "--username",
        username,
        "--configure",
        ".*",
        "--read",
        ".*",
        "--write",
        ".*",
    ]);

    run_succeeds(["users", "permissions"]).stdout(output_includes(username));

    run_succeeds(["permissions", "delete", "--username", username]);
    run_succeeds(["users", "permissions"]).stdout(output_includes(username).not());
    run_succeeds(["delete", "user", "--name", username]);

    Ok(())
}

#[test]
fn test_permissions_with_separate_auth_and_target_usernames() -> Result<(), Box<dyn Error>> {
    let target = "user_perms_separate_auth";
    let password = "pa$$w0rd";
    run_succeeds(["declare", "user", "--name", target, "--password", password]);

    // Global --username is for API auth, per-command --username is the permissions target
    run_succeeds([
        "--username",
        "guest",
        "--password",
        "guest",
        "permissions",
        "declare",
        "--username",
        target,
        "--configure",
        ".*",
        "--read",
        ".*",
        "--write",
        ".*",
    ]);

    run_succeeds(["permissions", "list"]).stdout(output_includes(target));

    run_succeeds([
        "--username",
        "guest",
        "--password",
        "guest",
        "permissions",
        "delete",
        "--username",
        target,
    ]);
    run_succeeds(["permissions", "list"]).stdout(output_includes(target).not());
    run_succeeds(["delete", "user", "--name", target]);

    Ok(())
}
