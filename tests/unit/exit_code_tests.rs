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

use rabbitmqadmin::exit_code::{EXIT_CODE_REFERENCE, Outcome, PARTIAL_SUCCESS_EXIT_CODE};
use sysexits::ExitCode;

#[test]
fn partial_success_constant_is_three() {
    assert_eq!(PARTIAL_SUCCESS_EXIT_CODE, 3);
}

#[test]
fn partial_success_does_not_collide_with_sysexits() {
    for code in [
        ExitCode::Usage,
        ExitCode::DataErr,
        ExitCode::Software,
        ExitCode::Unavailable,
        ExitCode::TempFail,
    ] {
        assert_ne!(
            PARTIAL_SUCCESS_EXIT_CODE,
            u8::from(code),
            "{:?} must not collide with the partial-success code",
            code
        );
    }
}

#[test]
fn outcome_success_is_zero() {
    assert_eq!(Outcome::Success.as_u8(), 0);
    assert!(Outcome::Success.is_success());
    assert!(!Outcome::Success.is_partial_success());
    assert!(!Outcome::Success.is_failure());
}

#[test]
fn outcome_partial_success_is_three() {
    assert_eq!(Outcome::PartialSuccess.as_u8(), PARTIAL_SUCCESS_EXIT_CODE);
    assert!(!Outcome::PartialSuccess.is_success());
    assert!(Outcome::PartialSuccess.is_partial_success());
    assert!(!Outcome::PartialSuccess.is_failure());
}

#[test]
fn outcome_failure_carries_exit_code() {
    let o = Outcome::Failure(ExitCode::DataErr);
    assert_eq!(o.as_u8(), u8::from(ExitCode::DataErr));
    assert!(!o.is_success());
    assert!(!o.is_partial_success());
    assert!(o.is_failure());
}

#[test]
fn exit_code_ok_round_trips_to_outcome_success() {
    assert_eq!(Outcome::from(ExitCode::Ok), Outcome::Success);
}

#[test]
fn exit_code_non_ok_round_trips_to_outcome_failure() {
    for ec in [
        ExitCode::Usage,
        ExitCode::DataErr,
        ExitCode::Unavailable,
        ExitCode::TempFail,
        ExitCode::Software,
    ] {
        assert_eq!(Outcome::from(ec), Outcome::Failure(ec));
    }
}

#[test]
fn outcome_to_process_exit_code() {
    // Round-trip via process::ExitCode is best validated by exit-code
    // value, not by ExitCode equality (which is opaque). We assert the
    // as_u8 mapping; the From impl just wraps process::ExitCode::from.
    assert_eq!(Outcome::Success.as_u8(), 0u8);
    assert_eq!(Outcome::PartialSuccess.as_u8(), 3u8);
    assert_eq!(Outcome::Failure(ExitCode::Usage).as_u8(), 64u8);
}

#[test]
fn exit_code_reference_table_is_well_formed() {
    assert!(!EXIT_CODE_REFERENCE.is_empty());
    let mut codes: Vec<u8> = EXIT_CODE_REFERENCE.iter().map(|(c, _)| *c).collect();
    codes.sort();
    codes.dedup();
    // Every entry must be unique.
    assert_eq!(codes.len(), EXIT_CODE_REFERENCE.len());
    // Must include 0, 3, and at least one sysexits code.
    assert!(codes.contains(&0));
    assert!(codes.contains(&PARTIAL_SUCCESS_EXIT_CODE));
    assert!(codes.contains(&u8::from(ExitCode::DataErr)));
}
