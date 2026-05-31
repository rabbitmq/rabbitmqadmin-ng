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

//! Generic scaffolding for bulk operations (delete a set of queues,
//! delete a set of virtual hosts, …) that need consistent
//! success, failure, skipped item/op accounting and reporting.

use crate::output::ProgressReporter;
use serde::Serialize;
use std::fmt;

/// Why a particular item was not acted upon. Skipped items do not
/// count toward partial-success calculation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SkipReason {
    /// The entity is protected from this kind of operation (e.g. the
    /// default virtual host). Carries a short human-readable reason.
    Protected { reason: String },
    /// The entity was not present at the time of the action, and the
    /// caller asked for idempotent behavior.
    AlreadyAbsent,
}

impl fmt::Display for SkipReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkipReason::Protected { reason } => write!(f, "protected: {reason}"),
            SkipReason::AlreadyAbsent => write!(f, "already absent"),
        }
    }
}

/// One per-item outcome from a bulk operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum ItemOutcome {
    Succeeded,
    Failed { error: String },
    Skipped { reason: SkipReason },
}

/// One named item plus its outcome. The `name` is what gets shown to
/// the user and serialized to JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BulkItem {
    pub name: String,
    #[serde(flatten)]
    pub outcome: ItemOutcome,
}

/// Aggregate result of a bulk operation.
///
/// Carries the raw item payloads (so the dispatch layer can render a
/// table of "what would be deleted" for `--dry-run`) alongside the
/// per-item outcomes and an `is_dry_run` flag. The classification
/// helpers (`is_full_success`, `is_partial`, etc.) drive the exit
/// code selection in the dispatch layer.
#[derive(Debug, Clone, Serialize)]
pub struct BulkReport<T> {
    pub items: Vec<T>,
    pub results: Vec<BulkItem>,
    pub is_dry_run: bool,
}

impl<T> BulkReport<T> {
    /// Create a report that represents the "preview" of a dry-run.
    /// No actions have been taken; `results` is empty.
    pub fn dry_run(items: Vec<T>) -> Self {
        Self {
            items,
            results: Vec::new(),
            is_dry_run: true,
        }
    }

    /// Create an empty report (no items matched). Not the same as a
    /// dry-run: this is a real execution that found nothing to do.
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            results: Vec::new(),
            is_dry_run: false,
        }
    }

    pub fn succeeded_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.outcome, ItemOutcome::Succeeded))
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.outcome, ItemOutcome::Failed { .. }))
            .count()
    }

    #[allow(dead_code)]
    pub fn skipped_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.outcome, ItemOutcome::Skipped { .. }))
            .count()
    }

    pub fn failures(&self) -> impl Iterator<Item = (&str, &str)> {
        self.results.iter().filter_map(|r| match &r.outcome {
            ItemOutcome::Failed { error } => Some((r.name.as_str(), error.as_str())),
            _ => None,
        })
    }

    pub fn skips(&self) -> impl Iterator<Item = (&str, &SkipReason)> {
        self.results.iter().filter_map(|r| match &r.outcome {
            ItemOutcome::Skipped { reason } => Some((r.name.as_str(), reason)),
            _ => None,
        })
    }

    /// True when this was a real execution that found no items to act on.
    pub fn nothing_to_do(&self) -> bool {
        !self.is_dry_run && self.results.is_empty()
    }

    /// True when every attempted item either succeeded or was skipped.
    /// Skips do not count as failures (per the SkipReason design).
    pub fn is_full_success(&self) -> bool {
        !self.is_dry_run && !self.results.is_empty() && self.failed_count() == 0
    }

    /// True when at least one item succeeded and at least one failed.
    /// Skipped items do not move the needle either direction.
    #[allow(dead_code)]
    pub fn is_partial(&self) -> bool {
        !self.is_dry_run && self.succeeded_count() > 0 && self.failed_count() > 0
    }

    /// True when at least one item failed and none succeeded. (Skips
    /// without any successes still count as total failure for the
    /// purpose of exit-code selection.)
    pub fn is_full_failure(&self) -> bool {
        !self.is_dry_run && self.succeeded_count() == 0 && self.failed_count() > 0
    }
}

/// Execution mode for [`bulk_op`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BulkMode {
    /// Try every item, accumulate results.
    ContinueOnError,
    /// Stop at the first failure (any subsequent items are not
    /// attempted and do not appear in the report).
    FailFast,
}

/// What an action closure may decide for a single item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemAction {
    /// The action ran and succeeded.
    Ok,
    /// The action determined the item should be skipped without
    /// being attempted (e.g. it's protected, or `--idempotently`
    /// turned an absent target into a no-op).
    Skip(SkipReason),
    /// The action ran and failed; the string is shown to the user.
    Fail(String),
}

/// Execute `action` over `items`, reporting progress and accumulating
/// per-item outcomes.
///
/// Contract: this helper calls exactly one of `report_success`,
/// `report_failure`, `report_skip` per item. `report_progress` is
/// **not** invoked by this helper; callers that want per-tick
/// updates should use it from within `action` if needed.
///
/// Under [`BulkMode::FailFast`] the loop stops at the first
/// `ItemAction::Fail`; items past that point are present in
/// `BulkReport.items` but absent from `BulkReport.results`.
pub fn bulk_op<T, N, F>(
    items: Vec<T>,
    mode: BulkMode,
    item_name: N,
    mut action: F,
    prog_rep: &mut dyn ProgressReporter,
    op_name: &str,
) -> BulkReport<T>
where
    N: Fn(&T) -> String,
    F: FnMut(&T) -> ItemAction,
{
    let total = items.len();
    let mut results = Vec::with_capacity(total);

    if total == 0 {
        return BulkReport::empty();
    }

    prog_rep.start_operation(total, op_name);

    for item in &items {
        let name = item_name(item);
        match action(item) {
            ItemAction::Ok => {
                prog_rep.report_success(&name);
                results.push(BulkItem {
                    name,
                    outcome: ItemOutcome::Succeeded,
                });
            }
            ItemAction::Skip(reason) => {
                prog_rep.report_skip(&name, &reason.to_string());
                results.push(BulkItem {
                    name,
                    outcome: ItemOutcome::Skipped { reason },
                });
            }
            ItemAction::Fail(error) => {
                prog_rep.report_failure(&name, &error);
                results.push(BulkItem {
                    name,
                    outcome: ItemOutcome::Failed { error },
                });
                if matches!(mode, BulkMode::FailFast) {
                    break;
                }
            }
        }
    }

    prog_rep.finish_operation(total);

    BulkReport {
        items,
        results,
        is_dry_run: false,
    }
}
