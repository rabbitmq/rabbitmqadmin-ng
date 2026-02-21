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

use crate::skip_if_rabbitmq_version_below;
use crate::test_helpers::*;
use std::error::Error;

#[test]
fn test_list_all_deprecated_features() -> Result<(), Box<dyn Error>> {
    skip_if_rabbitmq_version_below!(3, 13, 0);

    run_succeeds(["deprecated_features", "list"]).stdout(output_includes("ram_node_type"));

    Ok(())
}

#[test]
fn test_list_all_deprecated_features_via_alias() -> Result<(), Box<dyn Error>> {
    skip_if_rabbitmq_version_below!(3, 13, 0);

    run_succeeds(["list", "deprecated_features"]).stdout(output_includes("ram_node_type"));

    Ok(())
}
