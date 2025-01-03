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
fn test_memory_breakdown_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    let rc = api_client();
    let nodes = rc.list_nodes()?;
    let first = nodes.first().unwrap();

    run_succeeds(["show", "memory_breakdown", "--node", first.name.as_str()]).stdout(
        predicates::str::contains("Allocated but unused")
            .and(predicates::str::contains("Quorum queue ETS tables"))
            .and(predicates::str::contains("Client connections"))
            .and(predicates::str::contains("Metadata store")),
    );

    Ok(())
}
