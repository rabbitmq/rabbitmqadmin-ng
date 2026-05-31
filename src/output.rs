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
use crate::bulk::BulkReport;
use crate::columns;
use crate::config::SharedSettings;
use crate::errors::CommandRunError;
use crate::exit_code::Outcome;
use crate::tables;
use bel7_cli::Padding;
use clap::ArgMatches;
use indicatif::{ProgressBar, ProgressStyle};
use rabbitmq_http_client::password_hashing::HashingError;
use rabbitmq_http_client::responses::{
    NodeMemoryBreakdown, Overview, SchemaDefinitionSyncStatus, WarmStandbyReplicationStatus,
};
use serde::Serialize;
use std::fmt;
use sysexits::ExitCode;
use tabled::settings::object::{Rows, Segment};
use tabled::settings::{Format, Modify, Panel, Remove};
use tabled::{Table, Tabled};

pub use bel7_cli::TableStyle;

/// Output format selector for bulk operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BulkOutputFormat {
    /// Human-readable table on stdout for dry-run; stderr text for
    /// the live-run summary (default).
    #[default]
    Table,
    /// Pretty-printed JSON envelope on stdout (machine-readable).
    Json,
}

impl BulkOutputFormat {
    pub fn parse(s: Option<&str>) -> Self {
        match s.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
            Some("json") => BulkOutputFormat::Json,
            _ => BulkOutputFormat::Table,
        }
    }
}

/// Options that drive how a bulk report is rendered and how its
/// outcome is mapped to an exit code.
#[derive(Debug, Clone, Copy, Default)]
pub struct BulkReportOpts {
    /// Treat partial success as failure (exit `DataErr`).
    pub strict: bool,
    /// Opt-in: surface partial success as exit code `3`.
    /// Off by default for backwards compatibility — the CLI keeps
    /// returning `0` on partial success unless the caller asks for
    /// the richer behavior.
    pub detailed_exit_codes: bool,
    pub output_format: BulkOutputFormat,
}

/// Implemented by payload types that bulk operations may want to
/// preview in `--dry-run`. Decouples the dispatch layer from the
/// table-rendering layer.
pub trait BulkPreviewRow {
    fn preview_name(&self) -> String;
}

/// Map a [`BulkReport`] plus [`BulkReportOpts`] to a process [`Outcome`].
///
/// Pure: no I/O, no `self`, no `ResultHandler`. The decision matrix
/// below is the single place that owns "what exit code does this
/// invocation produce?".
///
/// | report state    | strict | detailed_exit_codes | outcome           |
/// | --------------- | ------ | ------------------- | ----------------- |
/// | dry-run         | any    | any                 | Success           |
/// | nothing matched | any    | any                 | Success           |
/// | full success    | any    | any                 | Success           |
/// | full failure    | any    | any                 | Failure(DataErr)  |
/// | partial         | true   | any                 | Failure(DataErr)  |
/// | partial         | false  | false               | Success (legacy)  |
/// | partial         | false  | true                | PartialSuccess    |
pub fn classify_bulk_outcome<T>(report: &BulkReport<T>, opts: BulkReportOpts) -> Outcome {
    if report.is_dry_run || report.nothing_to_do() || report.is_full_success() {
        return Outcome::Success;
    }
    if report.is_full_failure() {
        return Outcome::Failure(ExitCode::DataErr);
    }
    // Partial: at least one success, at least one failure.
    if opts.strict {
        return Outcome::Failure(ExitCode::DataErr);
    }
    if !opts.detailed_exit_codes {
        return Outcome::Success;
    }
    Outcome::PartialSuccess
}

#[derive(Tabled)]
struct DryRunRow {
    name: String,
}

// Wire envelope for --output json. Borrows the per-item results from
// the live BulkReport to avoid cloning the Vec.
#[derive(Serialize)]
struct BulkReportEnvelope<'a> {
    dry_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    preview: Option<Vec<String>>,
    attempted: usize,
    succeeded: usize,
    failed: usize,
    skipped: usize,
    results: &'a [crate::bulk::BulkItem],
}

type CommandResult<T> = Result<T, CommandRunError>;

#[derive(Copy, Clone)]
pub struct TableStyler {
    pub style: TableStyle,
    pub non_interactive: bool,
}

impl TableStyler {
    pub fn new(args: &SharedSettings) -> Self {
        if args.non_interactive {
            return Self {
                style: TableStyle::Borderless,
                non_interactive: true,
            };
        };

        Self {
            style: args.table_style.unwrap_or_default(),
            non_interactive: false,
        }
    }

    pub fn apply(self, table: &mut Table) {
        self.style.apply(table);

        if self.non_interactive {
            table.with(Padding::new(0, 1, 0, 0));
            table.with(Remove::row(Rows::first()));
            table.with(Modify::new(Segment::all()).with(Format::content(|s| s.replace('\n', ","))));
        }
    }
}

#[allow(dead_code)]
pub struct ResultHandler<'a> {
    cli_args: &'a SharedSettings,
    table_styler: TableStyler,
    pub non_interactive: bool,
    pub quiet: bool,
    pub idempotently: bool,
    pub exit_code: Option<ExitCode>,
    pub outcome: Option<Outcome>,
}

impl<'a> ResultHandler<'a> {
    pub fn new(common_args: &'a SharedSettings, command_args: &ArgMatches) -> Self {
        let non_interactive = common_args.non_interactive;
        let quiet = common_args.quiet;
        let idempotently = command_args
            .try_get_one::<bool>("idempotently")
            .ok()
            .flatten()
            .copied()
            .unwrap_or(false);

        let table_styler = TableStyler::new(common_args);

        Self {
            cli_args: common_args,
            table_styler,
            quiet,
            non_interactive,
            idempotently,
            exit_code: None,
            outcome: None,
        }
    }

    /// Compute the final outcome of this invocation. Prefers an
    /// explicitly-set `outcome` (the new path); falls back to
    /// mapping `exit_code` (the legacy path used by everything that
    /// did not opt in to the new shape). If neither is set, returns
    /// `Failure(fallback)` to preserve the historical
    /// `unwrap_or(ExitCode::DataErr)` defensive behavior of `main`.
    pub fn final_outcome_or(&self, fallback: ExitCode) -> Outcome {
        if let Some(o) = self.outcome {
            return o;
        }
        match self.exit_code {
            Some(e) => Outcome::from(e),
            None => Outcome::Failure(fallback),
        }
    }

    #[allow(dead_code)]
    pub fn instantiate_progress_reporter(&self) -> Box<dyn ProgressReporter> {
        match (self.quiet, self.non_interactive) {
            (true, _) => Box::new(QuietProgressReporter::new()),
            (false, true) => Box::new(NonInteractiveProgressReporter::new()),
            (false, false) => Box::new(InteractiveProgressReporter::new()),
        }
    }

    fn print_styled_table(&self, table: &mut Table) {
        self.table_styler.apply(table);
        println!("{}", table);
    }

    fn handle_table_result<T, F>(&mut self, result: CommandResult<T>, table_builder: F)
    where
        F: FnOnce(T) -> Table,
    {
        match result {
            Ok(data) => {
                self.exit_code = Some(ExitCode::Ok);
                let mut table = table_builder(data);
                self.print_styled_table(&mut table);
            }
            Err(error) => self.report_pre_command_run_error(&error),
        }
    }

    pub fn show_overview(&mut self, result: CommandResult<Overview>) {
        self.handle_table_result(result, tables::overview);
    }

    pub fn show_churn(&mut self, result: CommandResult<Overview>) {
        self.handle_table_result(result, tables::churn_overview);
    }

    pub fn show_salted_and_hashed_value(&mut self, result: Result<String, HashingError>) {
        match result {
            Ok(value) => {
                self.exit_code = Some(ExitCode::Ok);
                let mut table = tables::show_salted_and_hashed_value(value);
                self.print_styled_table(&mut table);
            }
            Err(error) => self.report_hashing_error(&error),
        }
    }

    pub fn tabular_result<T>(&mut self, result: CommandResult<Vec<T>>)
    where
        T: fmt::Debug + Tabled,
    {
        self.handle_table_result(result, Table::new);
    }

    pub fn tabular_result_with_columns<T>(
        &mut self,
        result: CommandResult<Vec<T>>,
        columns_arg: Option<String>,
    ) where
        T: fmt::Debug + Tabled,
    {
        match columns_arg {
            Some(cols) => {
                let column_list = columns::parse_columns(&cols);
                self.handle_table_result(result, |data| {
                    columns::build_table_with_columns(&data, &column_list)
                });
            }
            None => self.tabular_result(result),
        }
    }

    pub fn single_item_tabular_result_with_columns<T>(
        &mut self,
        result: CommandResult<T>,
        columns_arg: Option<String>,
    ) where
        T: fmt::Debug + Tabled,
    {
        match columns_arg {
            Some(cols) => {
                let column_list = columns::parse_columns(&cols);
                self.handle_table_result(result, |data| {
                    columns::build_table_with_columns(&[data], &column_list)
                });
            }
            None => self.handle_table_result(result, |data| Table::new([data])),
        }
    }

    pub fn single_value_output_with_result<T: fmt::Display>(
        &mut self,
        result: Result<T, CommandRunError>,
    ) {
        match result {
            Ok(output) => {
                self.exit_code = Some(ExitCode::Ok);
                println!("{}", output)
            }
            Err(error) => self.report_pre_command_run_error(&error),
        }
    }

    pub fn memory_breakdown_in_bytes_result(
        &mut self,
        result: CommandResult<Option<NodeMemoryBreakdown>>,
    ) {
        match result {
            Ok(Some(output)) => {
                self.exit_code = Some(ExitCode::Ok);
                let mut table = tables::memory_breakdown_in_bytes(output);
                self.print_styled_table(&mut table);
            }
            Ok(None) => {
                self.exit_code = Some(ExitCode::Ok);
                let mut table = tables::memory_breakdown_not_available();
                self.print_styled_table(&mut table);
            }
            Err(error) => self.report_pre_command_run_error(&error),
        }
    }

    pub fn memory_breakdown_in_percent_result(
        &mut self,
        result: CommandResult<Option<NodeMemoryBreakdown>>,
    ) {
        match result {
            Ok(Some(output)) => {
                self.exit_code = Some(ExitCode::Ok);
                let mut table = tables::memory_breakdown_in_percent(output);
                self.print_styled_table(&mut table);
            }
            Ok(None) => {
                self.exit_code = Some(ExitCode::Ok);
                let mut table = tables::memory_breakdown_not_available();
                self.print_styled_table(&mut table);
            }
            Err(error) => self.report_pre_command_run_error(&error),
        }
    }

    pub fn schema_definition_sync_status_result(
        &mut self,
        result: CommandResult<SchemaDefinitionSyncStatus>,
    ) {
        self.handle_table_result(result, tables::schema_definition_sync_status);
    }

    pub fn warm_standby_replication_status_result(
        &mut self,
        result: CommandResult<WarmStandbyReplicationStatus>,
    ) {
        match result {
            Ok(data) => {
                self.exit_code = Some(ExitCode::Ok);
                let tb = Table::builder(data.virtual_hosts);
                let mut table = tb.build();
                table.with(Panel::header("Warm Standby Replication Status"));
                self.print_styled_table(&mut table);
            }
            Err(error) => self.report_pre_command_run_error(&error),
        }
    }

    pub fn no_output_on_success<T>(&mut self, result: CommandResult<T>) {
        match result {
            Ok(_) => {
                self.exit_code = Some(ExitCode::Ok);
            }
            Err(error) => self.report_pre_command_run_error(&error),
        }
    }

    pub fn delete_operation_result<T>(&mut self, result: CommandResult<T>) {
        match result {
            Ok(_) => {
                self.exit_code = Some(ExitCode::Ok);
            }
            Err(ref error) => {
                let is_not_found = matches!(error, CommandRunError::NotFound);

                if is_not_found && self.idempotently {
                    self.exit_code = Some(ExitCode::Ok)
                } else {
                    self.report_pre_command_run_error(error)
                }
            }
        }
    }

    pub fn health_check_result(&mut self, result: CommandResult<()>) {
        match result {
            Ok(_) => {
                self.exit_code = Some(ExitCode::Ok);
                if !self.quiet {
                    println!("health check passed");
                }
            }
            Err(CommandRunError::HealthCheckFailed(ref info)) => {
                self.exit_code = Some(ExitCode::Unavailable);

                let mut table = tables::health_check_failure(
                    &info.health_check_path,
                    info.status_code,
                    info.details.clone(),
                );
                self.table_styler.apply(&mut table);

                println!("{}", table);
            }
            Err(ref e) => {
                eprintln!("{}", e);
                self.exit_code = Some(ExitCode::Unavailable);
            }
        }
    }

    /// Options controlling how a [`BulkReport`] is rendered and how
    /// its outcome is mapped to a process exit code. Passed in from
    /// the dispatch layer once per bulk-delete subcommand.
    pub fn render_bulk_report<T>(&mut self, report: BulkReport<T>, opts: BulkReportOpts)
    where
        T: BulkPreviewRow,
    {
        match opts.output_format {
            BulkOutputFormat::Json => self.render_bulk_report_json(&report),
            BulkOutputFormat::Table => self.render_bulk_report_table(&report),
        }
        self.classify_bulk_report(&report, opts);
    }

    fn render_bulk_report_json<T>(&self, report: &BulkReport<T>)
    where
        T: BulkPreviewRow,
    {
        // Decouple the wire schema from BulkReport<T>'s payload
        // (T may not be Serialize, and we want a stable schema).
        // Compute counts once to keep the linear-scan helpers off the
        // hot path.
        let mut succeeded = 0usize;
        let mut failed = 0usize;
        let mut skipped = 0usize;
        for item in &report.results {
            match item.outcome {
                crate::bulk::ItemOutcome::Succeeded => succeeded += 1,
                crate::bulk::ItemOutcome::Failed { .. } => failed += 1,
                crate::bulk::ItemOutcome::Skipped { .. } => skipped += 1,
            }
        }
        let preview = if report.is_dry_run {
            Some(
                report
                    .items
                    .iter()
                    .map(BulkPreviewRow::preview_name)
                    .collect(),
            )
        } else {
            None
        };
        let envelope = BulkReportEnvelope {
            dry_run: report.is_dry_run,
            preview,
            attempted: report.results.len(),
            succeeded,
            failed,
            skipped,
            results: &report.results,
        };
        match serde_json::to_string_pretty(&envelope) {
            Ok(json) => println!("{}", json),
            Err(err) => eprintln!("failed to serialize bulk report: {}", err),
        }
    }

    fn render_bulk_report_table<T>(&self, report: &BulkReport<T>)
    where
        T: BulkPreviewRow,
    {
        if report.is_dry_run {
            if report.items.is_empty() {
                // Stay silent for backwards compatibility: scripts that
                // used to see no output here must keep seeing none.
                return;
            }
            // Tabled-friendly view: a single-column "name" preview.
            let rows: Vec<DryRunRow> = report
                .items
                .iter()
                .map(|i| DryRunRow {
                    name: i.preview_name(),
                })
                .collect();
            let mut table = Table::new(rows);
            self.table_styler.apply(&mut table);
            println!("{}", table);
            return;
        }

        if report.nothing_to_do() {
            // Silent for backwards compatibility.
            return;
        }

        // List individual skips and failures on stderr. The aggregate
        // pass-or-fail line is left to InteractiveProgressReporter so we
        // don't print two summaries; --output json carries the full
        // counts in a structured envelope.
        if !self.quiet {
            for (name, reason) in report.skips() {
                eprintln!("skipped {}: {}", name, reason);
            }
            for (name, error) in report.failures() {
                eprintln!("failed {}: {}", name, error);
            }
        }
    }

    fn classify_bulk_report<T>(&mut self, report: &BulkReport<T>, opts: BulkReportOpts) {
        let outcome = classify_bulk_outcome(report, opts);
        self.outcome = Some(outcome);
        // Mirror in exit_code so the legacy main() fallback path also
        // observes this outcome.
        self.exit_code = Some(match outcome {
            Outcome::Success | Outcome::PartialSuccess => ExitCode::Ok,
            Outcome::Failure(e) => e,
        });
    }

    pub fn report_pre_command_run_error(&mut self, error: &CommandRunError) {
        eprintln!("{}", error);
        let code = match error {
            CommandRunError::UnknownCommandTarget { .. } => ExitCode::Usage,
            CommandRunError::MissingRequiredArgument { .. } => ExitCode::Usage,
            CommandRunError::InvalidArgumentValue { .. } => ExitCode::Usage,
            CommandRunError::ConflictingOptions { .. } => ExitCode::Usage,
            CommandRunError::MissingOptions { .. } => ExitCode::Usage,
            CommandRunError::MissingArgumentValue { .. } => ExitCode::Usage,
            CommandRunError::UnsupportedArgumentValue { .. } => ExitCode::Usage,
            CommandRunError::InvalidBaseUri { .. } => ExitCode::Usage,
            CommandRunError::CertificateFileCouldNotBeLoaded1 { .. } => ExitCode::DataErr,
            CommandRunError::CertificateFileCouldNotBeLoaded2 { .. } => ExitCode::DataErr,
            CommandRunError::CertificateFileNotFound { .. } => ExitCode::DataErr,
            CommandRunError::CertificateFileEmpty { .. } => ExitCode::DataErr,
            CommandRunError::CertificateFileInvalidPem { .. } => ExitCode::DataErr,
            CommandRunError::PrivateKeyFileUnsupported { .. } => ExitCode::DataErr,
            CommandRunError::CertificateKeyMismatch { .. } => ExitCode::DataErr,
            CommandRunError::IoError { .. } => ExitCode::DataErr,
            CommandRunError::FailureDuringExecution { .. } => ExitCode::DataErr,
            CommandRunError::HttpClientBuildError { .. } => ExitCode::DataErr,
            CommandRunError::ClientError { .. } => ExitCode::DataErr,
            CommandRunError::ServerError { .. } => ExitCode::DataErr,
            CommandRunError::NotFound => ExitCode::DataErr,
            CommandRunError::InvalidHeaderValue { .. } => ExitCode::DataErr,
            CommandRunError::IncompatibleBody { .. } => ExitCode::DataErr,
            CommandRunError::RequestError { .. } => ExitCode::DataErr,
            CommandRunError::JsonParseError { .. } => ExitCode::DataErr,
            CommandRunError::Other => ExitCode::DataErr,
            // HealthCheckFailed is handled separately in health_check_result
            CommandRunError::HealthCheckFailed { .. } => ExitCode::Unavailable,
        };
        self.exit_code = Some(code);
    }

    pub fn local_tabular_result<T>(&mut self, result: Result<Vec<T>, CommandRunError>)
    where
        T: fmt::Debug + Tabled,
    {
        self.handle_table_result(result, Table::new);
    }

    pub fn local_no_output_on_success(&mut self, result: Result<(), CommandRunError>) {
        match result {
            Ok(_) => {
                self.exit_code = Some(ExitCode::Ok);
            }
            Err(error) => self.report_pre_command_run_error(&error),
        }
    }

    //
    // Implementation
    //

    fn report_hashing_error(&mut self, error: &HashingError) {
        let mut table = tables::hashing_error_details(error);
        self.table_styler.apply(&mut table);
        eprintln!("{}", table);
        self.exit_code = Some(ExitCode::DataErr);
    }
}

#[allow(dead_code)]
pub trait ProgressReporter {
    fn start_operation(&mut self, total: usize, operation_name: &str);
    fn report_progress(&mut self, current: usize, total: usize, item_name: &str);
    fn report_success(&mut self, item_name: &str);
    fn report_skip(&mut self, item_name: &str, reason: &str);
    fn report_failure(&mut self, item_name: &str, error: &str);
    fn finish_operation(&mut self, total: usize);
}

#[allow(dead_code)]
pub struct InteractiveProgressReporter {
    bar: Option<ProgressBar>,
    failures: usize,
}

impl Default for InteractiveProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl InteractiveProgressReporter {
    pub fn new() -> Self {
        Self {
            bar: None,
            failures: 0,
        }
    }
}

// Contract: the bulk helper calls exactly one of report_success,
// report_failure, or report_skip per item. Each of those advances
// the progress bar by one. report_progress is advisory and does not
// move the bar.
impl ProgressReporter for InteractiveProgressReporter {
    fn start_operation(&mut self, total: usize, operation_name: &str) {
        let bar = ProgressBar::new(total as u64);
        bar.set_style(
            ProgressStyle::with_template(
                "{msg} [{bar:40.yellow/red}] {pos}/{len} ({percent}%) {elapsed_precise}",
            )
            .unwrap(),
        );
        bar.set_message(operation_name.to_string());
        self.bar = Some(bar);
        self.failures = 0;
    }

    fn report_progress(&mut self, _current: usize, _total: usize, _item_name: &str) {
        // Advisory only; per-item terminal calls drive the bar.
    }

    fn report_success(&mut self, _item_name: &str) {
        if let Some(bar) = &self.bar {
            bar.inc(1);
        }
    }

    fn report_skip(&mut self, _item_name: &str, _reason: &str) {
        if let Some(bar) = &self.bar {
            bar.inc(1);
        }
    }

    fn report_failure(&mut self, _item_name: &str, _error: &str) {
        self.failures += 1;
        if let Some(bar) = &self.bar {
            bar.inc(1);
        }
    }

    fn finish_operation(&mut self, total: usize) {
        if let Some(bar) = &self.bar {
            bar.finish();

            let successes = total - self.failures;
            if self.failures == 0 {
                eprintln!("✅ Completed: {} items processed successfully", total);
            } else if successes == 0 {
                eprintln!("❌ Failed: All {} items failed to process", total);
            } else {
                eprintln!(
                    "⚠️  Completed with failures: {} succeeded, {} failed out of {} total",
                    successes, self.failures, total
                );
            }
        }
        self.bar = None;
    }
}

#[allow(dead_code)]
pub struct NonInteractiveProgressReporter {
    bar: Option<ProgressBar>,
}

impl Default for NonInteractiveProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl NonInteractiveProgressReporter {
    pub fn new() -> Self {
        Self { bar: None }
    }
}

impl ProgressReporter for NonInteractiveProgressReporter {
    fn start_operation(&mut self, total: usize, operation_name: &str) {
        let bar = ProgressBar::new(total as u64);
        bar.set_style(
            ProgressStyle::with_template("{msg}: {pos}/{len} [{elapsed_precise}]").unwrap(),
        );
        bar.set_message(operation_name.to_string());
        self.bar = Some(bar);
    }

    fn report_progress(&mut self, _current: usize, _total: usize, _item_name: &str) {
        // Advisory only; per-item terminal calls drive the bar.
    }

    fn report_success(&mut self, _item_name: &str) {
        if let Some(bar) = &self.bar {
            bar.inc(1);
        }
    }

    fn report_skip(&mut self, _item_name: &str, _reason: &str) {
        if let Some(bar) = &self.bar {
            bar.inc(1);
        }
    }

    fn report_failure(&mut self, _item_name: &str, _error: &str) {
        if let Some(bar) = &self.bar {
            bar.inc(1);
        }
    }

    fn finish_operation(&mut self, total: usize) {
        if let Some(bar) = &self.bar {
            bar.finish();
            eprintln!("Completed: {} items processed", total);
        }
        self.bar = None;
    }
}

#[allow(dead_code)]
pub struct QuietProgressReporter;

impl Default for QuietProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl QuietProgressReporter {
    pub fn new() -> Self {
        Self
    }
}

impl ProgressReporter for QuietProgressReporter {
    fn start_operation(&mut self, _total: usize, _operation_name: &str) {
        // Silent
    }

    fn report_progress(&mut self, _current: usize, _total: usize, _item_name: &str) {
        // Silent
    }

    fn report_success(&mut self, _item_name: &str) {
        // Silent
    }

    fn report_skip(&mut self, _item_name: &str, _reason: &str) {
        // Silent
    }

    fn report_failure(&mut self, _item_name: &str, _error: &str) {
        // Silent
    }

    fn finish_operation(&mut self, _total: usize) {
        // Silent
    }
}
