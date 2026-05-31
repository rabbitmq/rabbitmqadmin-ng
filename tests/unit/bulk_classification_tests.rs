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

use rabbitmqadmin::bulk::{BulkItem, BulkReport, ItemOutcome, SkipReason};
use rabbitmqadmin::exit_code::Outcome;
use rabbitmqadmin::output::{BulkOutputFormat, BulkReportOpts, classify_bulk_outcome};
use sysexits::ExitCode;

fn report_with(succeeded: usize, failed: usize, skipped: usize) -> BulkReport<&'static str> {
    let mut results = Vec::new();
    for i in 0..succeeded {
        results.push(BulkItem {
            name: format!("ok-{i}"),
            outcome: ItemOutcome::Succeeded,
        });
    }
    for i in 0..failed {
        results.push(BulkItem {
            name: format!("bad-{i}"),
            outcome: ItemOutcome::Failed {
                error: "boom".into(),
            },
        });
    }
    for i in 0..skipped {
        results.push(BulkItem {
            name: format!("skip-{i}"),
            outcome: ItemOutcome::Skipped {
                reason: SkipReason::AlreadyAbsent,
            },
        });
    }
    BulkReport {
        items: vec![],
        results,
        is_dry_run: false,
    }
}

fn opts(strict: bool, detailed: bool) -> BulkReportOpts {
    BulkReportOpts {
        strict,
        detailed_exit_codes: detailed,
        output_format: BulkOutputFormat::Table,
    }
}

#[test]
fn dry_run_is_always_success_regardless_of_flags() {
    let dry: BulkReport<&str> = BulkReport::dry_run(vec!["x", "y"]);
    assert_eq!(
        classify_bulk_outcome(&dry, opts(false, false)),
        Outcome::Success
    );
    assert_eq!(
        classify_bulk_outcome(&dry, opts(true, false)),
        Outcome::Success
    );
    assert_eq!(
        classify_bulk_outcome(&dry, opts(false, true)),
        Outcome::Success
    );
}

#[test]
fn empty_match_is_always_success_regardless_of_flags() {
    let empty: BulkReport<&str> = BulkReport::empty();
    assert_eq!(
        classify_bulk_outcome(&empty, opts(false, false)),
        Outcome::Success
    );
    assert_eq!(
        classify_bulk_outcome(&empty, opts(true, false)),
        Outcome::Success
    );
    assert_eq!(
        classify_bulk_outcome(&empty, opts(false, true)),
        Outcome::Success
    );
}

#[test]
fn full_success_is_always_success_regardless_of_flags() {
    let r = report_with(3, 0, 0);
    assert_eq!(
        classify_bulk_outcome(&r, opts(false, false)),
        Outcome::Success
    );
    assert_eq!(
        classify_bulk_outcome(&r, opts(true, false)),
        Outcome::Success
    );
    assert_eq!(
        classify_bulk_outcome(&r, opts(false, true)),
        Outcome::Success
    );
}

#[test]
fn full_failure_is_always_data_err_regardless_of_flags() {
    let r = report_with(0, 3, 0);
    let expected = Outcome::Failure(ExitCode::DataErr);
    assert_eq!(classify_bulk_outcome(&r, opts(false, false)), expected);
    assert_eq!(classify_bulk_outcome(&r, opts(true, false)), expected);
    assert_eq!(classify_bulk_outcome(&r, opts(false, true)), expected);
}

#[test]
fn partial_with_no_flags_is_legacy_success() {
    // Backwards-compatible default: partial collapses to exit 0 unless
    // the user opts in to detailed exit codes.
    let r = report_with(2, 1, 0);
    assert_eq!(
        classify_bulk_outcome(&r, opts(false, false)),
        Outcome::Success
    );
}

#[test]
fn partial_with_strict_is_data_err() {
    let r = report_with(2, 1, 0);
    assert_eq!(
        classify_bulk_outcome(&r, opts(true, false)),
        Outcome::Failure(ExitCode::DataErr)
    );
}

#[test]
fn partial_with_detailed_exit_codes_is_partial_success() {
    let r = report_with(2, 1, 0);
    assert_eq!(
        classify_bulk_outcome(&r, opts(false, true)),
        Outcome::PartialSuccess
    );
}

#[test]
fn all_skipped_with_detailed_exit_codes_is_success() {
    // Skip-only is "the user got what they wanted"; never partial.
    let r = report_with(0, 0, 3);
    assert_eq!(
        classify_bulk_outcome(&r, opts(false, true)),
        Outcome::Success
    );
    assert_eq!(
        classify_bulk_outcome(&r, opts(true, false)),
        Outcome::Success
    );
}

#[test]
fn skipped_alongside_success_is_full_success() {
    let r = report_with(2, 0, 1);
    assert_eq!(
        classify_bulk_outcome(&r, opts(false, true)),
        Outcome::Success
    );
}

#[test]
fn skipped_alongside_failure_alone_is_full_failure() {
    // No successes, only skipped + failed: classified as full failure.
    let r = report_with(0, 2, 1);
    assert_eq!(
        classify_bulk_outcome(&r, opts(false, true)),
        Outcome::Failure(ExitCode::DataErr)
    );
}

#[test]
fn bulk_output_format_parse_accepts_json_case_insensitively() {
    assert_eq!(
        BulkOutputFormat::parse(Some("json")),
        BulkOutputFormat::Json
    );
    assert_eq!(
        BulkOutputFormat::parse(Some("JSON")),
        BulkOutputFormat::Json
    );
    assert_eq!(
        BulkOutputFormat::parse(Some(" Json ")),
        BulkOutputFormat::Json
    );
}

#[test]
fn bulk_output_format_parse_defaults_to_table() {
    assert_eq!(BulkOutputFormat::parse(None), BulkOutputFormat::Table);
    assert_eq!(
        BulkOutputFormat::parse(Some("table")),
        BulkOutputFormat::Table
    );
    assert_eq!(
        BulkOutputFormat::parse(Some("anything")),
        BulkOutputFormat::Table
    );
}

#[test]
fn bulk_report_opts_default_is_legacy_behavior() {
    let opts = BulkReportOpts::default();
    assert!(!opts.strict);
    assert!(!opts.detailed_exit_codes);
    assert_eq!(opts.output_format, BulkOutputFormat::Table);
}
