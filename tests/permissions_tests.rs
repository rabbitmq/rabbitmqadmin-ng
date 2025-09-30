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
use std::error::Error;
mod test_helpers;
use crate::test_helpers::*;

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
        "--user",
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

    run_succeeds(["permissions", "delete", "--user", username]);
    run_succeeds(["permissions", "list"]).stdout(output_includes(username).not());
    run_succeeds(["delete", "user", "--name", username]);

    Ok(())
}
