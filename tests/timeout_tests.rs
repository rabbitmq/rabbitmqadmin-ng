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
use std::error::Error;
use test_helpers::{run_fails, run_succeeds};

#[test]
fn timeout_flag_with_valid_value() -> Result<(), Box<dyn Error>> {
    run_succeeds(["--timeout", "30", "show", "overview"]);

    Ok(())
}

#[test]
fn timeout_flag_with_zero_value_should_fail() -> Result<(), Box<dyn Error>> {
    run_fails(["--timeout", "0", "show", "overview"]);

    Ok(())
}

#[test]
fn timeout_flag_with_negative_value_should_fail() -> Result<(), Box<dyn Error>> {
    run_fails(["--timeout", "-1", "show", "overview"]);

    Ok(())
}

#[test]
fn timeout_uses_default_when_not_specified() -> Result<(), Box<dyn Error>> {
    // Should use the default timeout of 60 seconds but we have no
    // easy way of testing this. Welp.
    run_succeeds(["show", "overview"]);

    Ok(())
}
