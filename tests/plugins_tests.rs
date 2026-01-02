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

use std::error::Error;
mod test_helpers;
use crate::test_helpers::*;

#[test]
fn test_plugins_list_all_succeeds() -> Result<(), Box<dyn Error>> {
    run_succeeds(["plugins", "list_all"]).stdout(output_includes("rabbitmq_management"));

    Ok(())
}

#[test]
fn test_plugins_list_on_node_succeeds() -> Result<(), Box<dyn Error>> {
    let rc = api_client();
    let nodes = rc.list_nodes()?;
    let first = nodes.first().unwrap();

    run_succeeds(["plugins", "list_on_node", "--node", first.name.as_str()])
        .stdout(output_includes("rabbitmq_management"));

    Ok(())
}
