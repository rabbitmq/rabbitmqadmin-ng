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

use crate::test_helpers::*;
use std::error::Error;

#[test]
fn test_shell_exit_codes_prints_table() -> Result<(), Box<dyn Error>> {
    run_succeeds(["shell", "exit-codes"])
        .stdout(output_includes("success"))
        .stdout(output_includes("partial success"))
        .stdout(output_includes("usage error"))
        .stdout(output_includes("data error"));
    Ok(())
}

#[test]
fn test_shell_exit_codes_lists_zero_three_and_sixty_four() -> Result<(), Box<dyn Error>> {
    let assert = run_succeeds(["shell", "exit-codes"]);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    assert!(stdout.lines().any(|l| l.trim_start().starts_with("0 ")
        || l.trim_start().starts_with("0\t")
        || l.trim_start().starts_with("0  ")));
    assert!(stdout.lines().any(|l| l.trim_start().starts_with('3')));
    assert!(stdout.lines().any(|l| l.trim_start().starts_with("64 ")));
    Ok(())
}

#[test]
fn test_shell_exit_codes_help_is_available() -> Result<(), Box<dyn Error>> {
    run_succeeds(["shell", "exit-codes", "--help"]).stdout(output_includes("exit codes"));
    Ok(())
}
