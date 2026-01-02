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

mod test_helpers;
use crate::test_helpers::*;
use std::error::Error;
#[test]
fn test_list_connections1() -> Result<(), Box<dyn Error>> {
    run_succeeds(["connections", "list"]);

    Ok(())
}

#[test]
fn test_list_connections2() -> Result<(), Box<dyn Error>> {
    run_succeeds(["list", "connections"]);

    Ok(())
}

#[test]
fn test_list_connections_table_styles() -> Result<(), Box<dyn Error>> {
    run_succeeds(["--table-style", "markdown", "list", "connections"]);

    Ok(())
}

#[test]
fn test_list_user_connections1() -> Result<(), Box<dyn Error>> {
    run_succeeds([
        "--table-style",
        "markdown",
        "connections",
        "list_of_user",
        "--username",
        "monitoring",
    ]);

    Ok(())
}

#[test]
fn test_list_user_connections2() -> Result<(), Box<dyn Error>> {
    run_succeeds([
        "--table-style",
        "markdown",
        "list",
        "user_connections",
        "--username",
        "monitoring",
    ]);

    Ok(())
}

#[test]
fn test_connections_close_idempotently() -> Result<(), Box<dyn Error>> {
    run_succeeds([
        "connections",
        "close",
        "--name",
        "non-existent-connection-12345",
        "--idempotently",
    ]);

    run_succeeds([
        "connections",
        "close_of_user",
        "--username",
        "non-existent-user-12345",
        "--idempotently",
    ]);

    Ok(())
}
