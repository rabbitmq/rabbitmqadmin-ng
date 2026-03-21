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

use crate::test_helpers::{output_includes, run_fails, run_succeeds};
use std::error::Error;

#[test]
fn test_verbose_logs_http_method_and_url_to_stderr() -> Result<(), Box<dyn Error>> {
    run_succeeds(["--verbose", "vhosts", "list"]).stderr(output_includes(
        "HTTP GET: http://localhost:15672/api/vhosts",
    ));

    Ok(())
}

#[test]
fn test_verbose_and_quiet_are_mutually_exclusive() -> Result<(), Box<dyn Error>> {
    run_fails(["--verbose", "--quiet", "vhosts", "list"])
        .stderr(output_includes("cannot be used with"));

    Ok(())
}
