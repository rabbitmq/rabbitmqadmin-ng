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

use crate::test_helpers::*;

#[test]
fn test_list_auth_attempts() -> Result<(), Box<dyn Error>> {
    let rc = api_client();
    let nodes = rc.list_nodes()?;
    let first = nodes.first().unwrap();

    run_succeeds(["auth_attempts", "stats", "--node", first.name.as_str()]).stdout(
        output_includes("Protocol")
            .and(output_includes("Number of attempts"))
            .and(output_includes("Successful"))
            .and(output_includes("Failed")),
    );

    Ok(())
}

#[test]
fn test_list_auth_attempts_requires_node_argument() -> Result<(), Box<dyn Error>> {
    run_fails(["auth_attempts", "stats"]).stderr(output_includes("--node"));

    Ok(())
}
