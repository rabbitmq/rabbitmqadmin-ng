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
use predicates::prelude::*;
use std::error::Error;

#[test]
fn test_list_feature_flags() -> Result<(), Box<dyn Error>> {
    skip_if_rabbitmq_version_below!(4, 0, 0);

    run_succeeds(["list", "feature_flags"])
        .stdout(output_includes("rabbitmq_4.0.0").and(output_includes("khepri_db")));

    Ok(())
}
