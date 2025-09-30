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
fn test_vhost_limits() -> Result<(), Box<dyn Error>> {
    let limit_name = "max-connections";
    run_succeeds([
        "vhost_limits",
        "declare",
        "--name",
        limit_name,
        "--value",
        "1234",
    ]);

    run_succeeds(["vhost_limits", "list"])
        .stdout(output_includes(limit_name).and(output_includes("1234")));
    run_succeeds(["vhost_limits", "delete", "--name", limit_name]);
    run_succeeds(["vhost_limits", "list"]).stdout(output_includes(limit_name).not());

    Ok(())
}
