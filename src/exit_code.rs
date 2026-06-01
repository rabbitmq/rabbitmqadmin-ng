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

//! Process exit codes used by rabbitmqadmin.
//!
//! Shape mirrors `bel7_cli::Outcome` / `bel7_cli::PARTIAL_SUCCESS_I32`
//! deliberately, but stays local so the crate does not need to pull in
//! a second sysexits major version (bel7-cli pins `sysexits = "0.11"`,
//! we use `sysexits = "0.13"` directly).

use std::process;
use sysexits::ExitCode;

/// Exit code for the partial-success outcome: a command ran but did
/// not complete every unit of work it was asked to.
///
/// Sits in the small-integer band next to `diff`'s exit `1` and
/// `rsync --partial`'s exit `23`, well below sysexits' `EX_USAGE = 64`
/// and above. Mirrors the value of `bel7_cli::PARTIAL_SUCCESS_I32`.
pub const PARTIAL_SUCCESS_EXIT_CODE: u8 = 3;

/// A producer-side outcome for a single rabbitmqadmin invocation.
///
/// Mirrors `bel7_cli::Outcome` in shape but carries a
/// `sysexits::ExitCode` directly (no `ExitCodeProvider` trait, no
/// `errors` feature) so the dependency graph stays single-version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Every unit of work completed.
    Success,
    /// Some units of work completed, others did not. Maps to
    /// [`PARTIAL_SUCCESS_EXIT_CODE`].
    PartialSuccess,
    /// The command failed. The wrapped `sysexits::ExitCode` selects
    /// the specific failure code.
    Failure(ExitCode),
}

impl Outcome {
    /// The numeric exit code this outcome corresponds to.
    pub fn as_u8(&self) -> u8 {
        match self {
            Outcome::Success => 0,
            Outcome::PartialSuccess => PARTIAL_SUCCESS_EXIT_CODE,
            Outcome::Failure(e) => u8::from(*e),
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Outcome::Success)
    }

    pub fn is_partial_success(&self) -> bool {
        matches!(self, Outcome::PartialSuccess)
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, Outcome::Failure(_))
    }
}

impl From<Outcome> for process::ExitCode {
    fn from(o: Outcome) -> Self {
        process::ExitCode::from(o.as_u8())
    }
}

impl From<ExitCode> for Outcome {
    fn from(e: ExitCode) -> Self {
        match e {
            ExitCode::Ok => Outcome::Success,
            other => Outcome::Failure(other),
        }
    }
}
