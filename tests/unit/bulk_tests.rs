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

use rabbitmqadmin::bulk::{BulkMode, BulkReport, ItemAction, ItemOutcome, SkipReason, bulk_op};
use rabbitmqadmin::output::ProgressReporter;
use std::cell::RefCell;

// A test-only reporter that records the sequence of calls so tests
// can assert the bulk_op contract: exactly one of report_success,
// report_failure, or report_skip per item, framed by start_operation
// and finish_operation.
#[derive(Default)]
struct RecordingReporter {
    calls: RefCell<Vec<String>>,
}

impl RecordingReporter {
    fn snapshot(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl ProgressReporter for RecordingReporter {
    fn start_operation(&mut self, total: usize, name: &str) {
        self.calls
            .borrow_mut()
            .push(format!("start:{name}:{total}"));
    }
    fn report_progress(&mut self, _c: usize, _t: usize, name: &str) {
        self.calls.borrow_mut().push(format!("progress:{name}"));
    }
    fn report_success(&mut self, name: &str) {
        self.calls.borrow_mut().push(format!("success:{name}"));
    }
    fn report_skip(&mut self, name: &str, reason: &str) {
        self.calls
            .borrow_mut()
            .push(format!("skip:{name}:{reason}"));
    }
    fn report_failure(&mut self, name: &str, error: &str) {
        self.calls
            .borrow_mut()
            .push(format!("failure:{name}:{error}"));
    }
    fn finish_operation(&mut self, total: usize) {
        self.calls.borrow_mut().push(format!("finish:{total}"));
    }
}

#[test]
fn empty_items_returns_empty_report() {
    let mut rep = RecordingReporter::default();
    let r: BulkReport<&str> = bulk_op(
        vec![],
        BulkMode::ContinueOnError,
        |s: &&str| s.to_string(),
        |_| ItemAction::Ok,
        &mut rep,
        "noop",
    );
    assert!(r.results.is_empty());
    assert!(r.nothing_to_do());
    assert!(!r.is_full_success());
    assert!(!r.is_partial());
    assert!(!r.is_full_failure());
    // No progress calls when there are no items.
    assert!(rep.snapshot().is_empty());
}

#[test]
fn all_ok_produces_full_success() {
    let mut rep = RecordingReporter::default();
    let r = bulk_op(
        vec!["a", "b", "c"],
        BulkMode::ContinueOnError,
        |s: &&str| s.to_string(),
        |_| ItemAction::Ok,
        &mut rep,
        "delete",
    );
    assert!(r.is_full_success());
    assert!(!r.nothing_to_do());
    assert_eq!(r.succeeded_count(), 3);
    assert_eq!(r.failed_count(), 0);
    assert_eq!(r.skipped_count(), 0);
}

#[test]
fn all_fail_produces_full_failure() {
    let mut rep = RecordingReporter::default();
    let r = bulk_op(
        vec!["a", "b"],
        BulkMode::ContinueOnError,
        |s: &&str| s.to_string(),
        |_| ItemAction::Fail("boom".into()),
        &mut rep,
        "delete",
    );
    assert!(r.is_full_failure());
    assert!(!r.is_full_success());
    assert!(!r.is_partial());
    assert_eq!(r.failed_count(), 2);
}

#[test]
fn mixed_produces_partial() {
    let mut rep = RecordingReporter::default();
    let r = bulk_op(
        vec!["ok", "bad", "also_ok"],
        BulkMode::ContinueOnError,
        |s: &&str| s.to_string(),
        |s| {
            if *s == "bad" {
                ItemAction::Fail("nope".into())
            } else {
                ItemAction::Ok
            }
        },
        &mut rep,
        "delete",
    );
    assert!(r.is_partial());
    assert!(!r.is_full_success());
    assert!(!r.is_full_failure());
    assert_eq!(r.succeeded_count(), 2);
    assert_eq!(r.failed_count(), 1);
}

#[test]
fn skipped_items_do_not_count_as_failures() {
    let mut rep = RecordingReporter::default();
    let r = bulk_op(
        vec!["a", "b", "c"],
        BulkMode::ContinueOnError,
        |s: &&str| s.to_string(),
        |s| {
            if *s == "b" {
                ItemAction::Skip(SkipReason::AlreadyAbsent)
            } else {
                ItemAction::Ok
            }
        },
        &mut rep,
        "delete",
    );
    assert!(r.is_full_success());
    assert!(!r.is_partial());
    assert!(!r.is_full_failure());
    assert_eq!(r.succeeded_count(), 2);
    assert_eq!(r.skipped_count(), 1);
    assert_eq!(r.failed_count(), 0);
}

#[test]
fn all_skipped_is_full_success_for_exit_code_purposes() {
    let mut rep = RecordingReporter::default();
    let r = bulk_op(
        vec!["a", "b"],
        BulkMode::ContinueOnError,
        |s: &&str| s.to_string(),
        |_| {
            ItemAction::Skip(SkipReason::Protected {
                reason: "test".into(),
            })
        },
        &mut rep,
        "delete",
    );
    // All skipped, no failures: the user got what they asked for
    // (idempotent delete where everything was already gone, or
    // every match was protected). Exit 0 is correct.
    assert!(r.is_full_success());
    assert!(!r.is_full_failure());
    assert!(!r.is_partial());
    assert_eq!(r.skipped_count(), 2);
    assert_eq!(r.failed_count(), 0);
    assert_eq!(r.succeeded_count(), 0);
}

#[test]
fn fail_fast_stops_at_first_failure() {
    let mut rep = RecordingReporter::default();
    let r = bulk_op(
        vec!["a", "boom", "c", "d"],
        BulkMode::FailFast,
        |s: &&str| s.to_string(),
        |s| {
            if *s == "boom" {
                ItemAction::Fail("x".into())
            } else {
                ItemAction::Ok
            }
        },
        &mut rep,
        "delete",
    );
    // a succeeded, boom failed, c and d were never attempted.
    assert_eq!(r.results.len(), 2);
    assert_eq!(r.succeeded_count(), 1);
    assert_eq!(r.failed_count(), 1);
    assert!(r.is_partial());
}

#[test]
fn results_preserve_input_order() {
    let mut rep = RecordingReporter::default();
    let r = bulk_op(
        vec!["a", "b", "c", "d"],
        BulkMode::ContinueOnError,
        |s: &&str| s.to_string(),
        |_| ItemAction::Ok,
        &mut rep,
        "delete",
    );
    let names: Vec<&str> = r.results.iter().map(|i| i.name.as_str()).collect();
    assert_eq!(names, vec!["a", "b", "c", "d"]);
}

#[test]
fn dry_run_constructor_carries_items_and_no_results() {
    let r: BulkReport<&str> = BulkReport::dry_run(vec!["x", "y"]);
    assert!(r.is_dry_run);
    assert!(!r.is_full_success());
    assert!(!r.is_full_failure());
    assert!(!r.is_partial());
    assert!(!r.nothing_to_do()); // dry_run with items is not "nothing"
    assert_eq!(r.items.len(), 2);
    assert!(r.results.is_empty());
}

#[test]
fn empty_constructor_signals_nothing_to_do() {
    let r: BulkReport<&str> = BulkReport::empty();
    assert!(!r.is_dry_run);
    assert!(r.nothing_to_do());
    assert!(!r.is_full_success());
    assert!(!r.is_full_failure());
    assert!(!r.is_partial());
}

#[test]
fn progress_reporter_contract_called_per_item() {
    // bulk_op must call exactly one of success, skip, or failure per
    // item, framed by start_operation and finish_operation. report_progress
    // is advisory and must not be invoked by bulk_op itself.
    let mut rep = RecordingReporter::default();
    let _ = bulk_op(
        vec!["ok", "skip", "fail"],
        BulkMode::ContinueOnError,
        |s: &&str| s.to_string(),
        |s| match *s {
            "ok" => ItemAction::Ok,
            "skip" => ItemAction::Skip(SkipReason::AlreadyAbsent),
            _ => ItemAction::Fail("boom".into()),
        },
        &mut rep,
        "delete",
    );
    let calls = rep.snapshot();
    assert_eq!(calls.first().unwrap(), "start:delete:3");
    assert_eq!(calls.last().unwrap(), "finish:3");
    // No advisory progress ticks from bulk_op itself.
    assert!(!calls.iter().any(|c| c.starts_with("progress:")));
    assert!(calls.iter().any(|c| c == "success:ok"));
    assert!(calls.iter().any(|c| c.starts_with("skip:skip:")));
    assert!(calls.iter().any(|c| c.starts_with("failure:fail:boom")));
}

#[test]
fn failures_iterator_yields_only_failures() {
    let mut rep = RecordingReporter::default();
    let r = bulk_op(
        vec!["a", "b", "c"],
        BulkMode::ContinueOnError,
        |s: &&str| s.to_string(),
        |s| {
            if *s == "b" {
                ItemAction::Fail("nope".into())
            } else {
                ItemAction::Ok
            }
        },
        &mut rep,
        "delete",
    );
    let failures: Vec<(&str, &str)> = r.failures().collect();
    assert_eq!(failures, vec![("b", "nope")]);
}

#[test]
fn skips_iterator_yields_only_skips() {
    let mut rep = RecordingReporter::default();
    let r = bulk_op(
        vec!["a", "b"],
        BulkMode::ContinueOnError,
        |s: &&str| s.to_string(),
        |s| {
            if *s == "a" {
                ItemAction::Skip(SkipReason::AlreadyAbsent)
            } else {
                ItemAction::Ok
            }
        },
        &mut rep,
        "delete",
    );
    let skips: Vec<(&str, &SkipReason)> = r.skips().collect();
    assert_eq!(skips.len(), 1);
    assert_eq!(skips[0].0, "a");
    assert!(matches!(skips[0].1, SkipReason::AlreadyAbsent));
}

#[test]
fn item_outcome_succeeded_serializes() {
    let item = rabbitmqadmin::bulk::BulkItem {
        name: "q1".into(),
        outcome: ItemOutcome::Succeeded,
    };
    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("\"name\":\"q1\""));
    assert!(json.contains("\"outcome\":\"succeeded\""));
}

#[test]
fn item_outcome_failed_serializes_error() {
    let item = rabbitmqadmin::bulk::BulkItem {
        name: "q1".into(),
        outcome: ItemOutcome::Failed {
            error: "bang".into(),
        },
    };
    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("\"outcome\":\"failed\""));
    assert!(json.contains("\"error\":\"bang\""));
}

#[test]
fn item_outcome_skipped_serializes_reason_kind() {
    let item = rabbitmqadmin::bulk::BulkItem {
        name: "q1".into(),
        outcome: ItemOutcome::Skipped {
            reason: SkipReason::AlreadyAbsent,
        },
    };
    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("\"outcome\":\"skipped\""));
    assert!(json.contains("\"kind\":\"already_absent\""));
}

#[test]
fn skip_reason_display_human_readable() {
    assert_eq!(
        SkipReason::AlreadyAbsent.to_string(),
        "already absent".to_string()
    );
    assert_eq!(
        SkipReason::Protected {
            reason: "default virtual host".into(),
        }
        .to_string(),
        "protected: default virtual host".to_string()
    );
}
