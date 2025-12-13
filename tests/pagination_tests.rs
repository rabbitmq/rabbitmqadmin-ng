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

use std::error::Error;

mod test_helpers;
use crate::test_helpers::*;

#[test]
fn test_list_users_with_pagination() -> Result<(), Box<dyn Error>> {
    run_succeeds(["list", "users", "--page", "1", "--page-size", "50"]);
    run_succeeds(["users", "list", "--page", "1", "--page-size", "50"]);

    Ok(())
}

#[test]
fn test_list_channels_with_pagination() -> Result<(), Box<dyn Error>> {
    run_succeeds(["list", "channels", "--page", "1", "--page-size", "50"]);
    run_succeeds(["channels", "list", "--page", "1", "--page-size", "50"]);

    Ok(())
}

#[test]
fn test_list_exchanges_with_pagination() -> Result<(), Box<dyn Error>> {
    run_succeeds(["list", "exchanges", "--page", "1", "--page-size", "50"]);
    run_succeeds(["exchanges", "list", "--page", "1", "--page-size", "50"]);

    Ok(())
}

#[test]
fn test_list_connections_with_pagination() -> Result<(), Box<dyn Error>> {
    run_succeeds(["list", "connections", "--page", "1", "--page-size", "50"]);

    Ok(())
}

#[test]
fn test_list_queues_with_pagination() -> Result<(), Box<dyn Error>> {
    run_succeeds(["list", "queues", "--page", "1", "--page-size", "50"]);
    run_succeeds(["queues", "list", "--page", "1", "--page-size", "50"]);

    Ok(())
}
