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

use proptest::prelude::*;
use rabbitmqadmin::bulk::{BulkMode, ItemAction, SkipReason, bulk_op};
use rabbitmqadmin::output::ProgressReporter;

#[derive(Clone, Copy, Debug)]
enum Verdict {
    Ok,
    Skip,
    Fail,
}

fn verdict_strategy() -> impl Strategy<Value = Verdict> {
    prop_oneof![Just(Verdict::Ok), Just(Verdict::Skip), Just(Verdict::Fail),]
}

struct NoopReporter;
impl ProgressReporter for NoopReporter {
    fn start_operation(&mut self, _: usize, _: &str) {}
    fn report_progress(&mut self, _: usize, _: usize, _: &str) {}
    fn report_success(&mut self, _: &str) {}
    fn report_skip(&mut self, _: &str, _: &str) {}
    fn report_failure(&mut self, _: &str, _: &str) {}
    fn finish_operation(&mut self, _: usize) {}
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// At-most-one-classification invariant: a non-dry-run report is in
    /// at most one of full_success, partial, or full_failure at a time.
    #[test]
    fn classifications_are_mutually_exclusive(verdicts in proptest::collection::vec(verdict_strategy(), 0..50)) {
        let items: Vec<usize> = (0..verdicts.len()).collect();
        let mut rep = NoopReporter;
        let r = bulk_op(
            items,
            BulkMode::ContinueOnError,
            |i| format!("item-{i}"),
            |i| match verdicts[*i] {
                Verdict::Ok => ItemAction::Ok,
                Verdict::Skip => ItemAction::Skip(SkipReason::AlreadyAbsent),
                Verdict::Fail => ItemAction::Fail("x".into()),
            },
            &mut rep,
            "p",
        );
        let classes = [r.is_full_success(), r.is_partial(), r.is_full_failure()];
        let true_count = classes.iter().filter(|b| **b).count();
        prop_assert!(true_count <= 1, "at most one classification may be true at a time");
    }

    /// Counts cover every result entry exactly once.
    #[test]
    fn counts_cover_all_results(verdicts in proptest::collection::vec(verdict_strategy(), 0..50)) {
        let items: Vec<usize> = (0..verdicts.len()).collect();
        let mut rep = NoopReporter;
        let r = bulk_op(
            items,
            BulkMode::ContinueOnError,
            |i| format!("i-{i}"),
            |i| match verdicts[*i] {
                Verdict::Ok => ItemAction::Ok,
                Verdict::Skip => ItemAction::Skip(SkipReason::AlreadyAbsent),
                Verdict::Fail => ItemAction::Fail("x".into()),
            },
            &mut rep,
            "p",
        );
        let sum = r.succeeded_count() + r.failed_count() + r.skipped_count();
        prop_assert_eq!(sum, r.results.len());
    }

    /// FailFast never produces more results than ContinueOnError, and
    /// stops at the first failure: at most one failed entry in the
    /// FailFast report.
    #[test]
    fn fail_fast_stops_at_first_failure(verdicts in proptest::collection::vec(verdict_strategy(), 0..50)) {
        let items: Vec<usize> = (0..verdicts.len()).collect();
        let mut rep = NoopReporter;
        let r = bulk_op(
            items,
            BulkMode::FailFast,
            |i| format!("i-{i}"),
            |i| match verdicts[*i] {
                Verdict::Ok => ItemAction::Ok,
                Verdict::Skip => ItemAction::Skip(SkipReason::AlreadyAbsent),
                Verdict::Fail => ItemAction::Fail("x".into()),
            },
            &mut rep,
            "p",
        );
        prop_assert!(r.failed_count() <= 1);
    }

    /// All-success input always classifies as full_success when
    /// non-empty.
    #[test]
    fn all_ok_is_full_success(n in 1usize..40) {
        let items: Vec<usize> = (0..n).collect();
        let mut rep = NoopReporter;
        let r = bulk_op(
            items,
            BulkMode::ContinueOnError,
            |i| format!("i-{i}"),
            |_| ItemAction::Ok,
            &mut rep,
            "p",
        );
        prop_assert!(r.is_full_success());
        prop_assert!(!r.is_partial());
        prop_assert!(!r.is_full_failure());
        prop_assert_eq!(r.succeeded_count(), n);
    }

    /// All-fail input always classifies as full_failure when non-empty.
    #[test]
    fn all_fail_is_full_failure(n in 1usize..40) {
        let items: Vec<usize> = (0..n).collect();
        let mut rep = NoopReporter;
        let r = bulk_op(
            items,
            BulkMode::ContinueOnError,
            |i| format!("i-{i}"),
            |_| ItemAction::Fail("nope".into()),
            &mut rep,
            "p",
        );
        prop_assert!(r.is_full_failure());
        prop_assert!(!r.is_partial());
        prop_assert!(!r.is_full_success());
        prop_assert_eq!(r.failed_count(), n);
    }

    /// Empty input is always classified as "nothing to do".
    #[test]
    fn empty_is_nothing_to_do(_seed in any::<u64>()) {
        let items: Vec<usize> = Vec::new();
        let mut rep = NoopReporter;
        let r = bulk_op(
            items,
            BulkMode::ContinueOnError,
            |i| format!("{i}"),
            |_| ItemAction::Ok,
            &mut rep,
            "p",
        );
        prop_assert!(r.nothing_to_do());
        prop_assert!(!r.is_full_success());
        prop_assert!(!r.is_partial());
        prop_assert!(!r.is_full_failure());
    }
}
