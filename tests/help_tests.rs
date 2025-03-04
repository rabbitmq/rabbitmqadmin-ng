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
use test_helpers::{run_fails, run_succeeds};

#[test]
fn show_help_with_no_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let args: [&str; 0] = [];
    run_fails(args).stderr(predicate::str::contains(
        "requires a subcommand but one was not provided",
    ));

    Ok(())
}

#[test]
fn show_subcommands_with_no_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let args: [&str; 0] = [];
    run_fails(args).stderr(predicate::str::contains("subcommands:"));

    Ok(())
}

#[test]
fn show_subcommands_with_category_name_and_help() -> Result<(), Box<dyn std::error::Error>> {
    let args = ["declare", "--help"];
    run_succeeds(args).stdout(predicate::str::contains("Commands:"));

    Ok(())
}

#[test]
fn shows_subcommand_specific_info_with_help() -> Result<(), Box<dyn std::error::Error>> {
    let args = ["declare", "queue", "--help"];
    run_succeeds(args).stdout(predicate::str::contains("Usage:"));

    Ok(())
}
