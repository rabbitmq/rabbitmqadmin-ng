// Copyright (C) 2023-2024 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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
fn test_enable_a_feature_flag() -> Result<(), Box<dyn std::error::Error>> {
    let ff_name = "detailed_queues_endpoint";

    run_succeeds(["feature_flags", "enable", "--name", ff_name]);
    run_succeeds(["feature_flags", "list"]).stdout(predicate::str::contains(ff_name));

    Ok(())
}

#[test]
fn test_enable_all_stable_feature_flags() -> Result<(), Box<dyn std::error::Error>> {
    let ff_name = "rabbitmq_4.0.0";

    run_succeeds(["feature_flags", "enable_all"]);
    run_succeeds(["feature_flags", "list"]).stdout(predicate::str::contains(ff_name));

    Ok(())
}
