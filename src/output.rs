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
use crate::config::SharedSettings;
use crate::errors::CommandRunError;
use crate::tables;
use clap::ArgMatches;
use rabbitmq_http_client::blocking_api::{HttpClientError, Result as ClientResult};
use rabbitmq_http_client::error::Error as ClientError;
use rabbitmq_http_client::password_hashing::HashingError;
use rabbitmq_http_client::responses::{
    NodeMemoryBreakdown, Overview, SchemaDefinitionSyncStatus, WarmStandbyReplicationStatus,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::fmt;
use sysexits::ExitCode;
use tabled::settings::object::Rows;

use tabled::settings::{Panel, Remove, Style};
use tabled::{
    Table, Tabled,
    settings::{Format, Modify, object::Segment},
};

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum TableStyle {
    #[default]
    Modern,
    Borderless,
    Markdown,
    Sharp,
    Ascii,
    Psql,
    Dots,
}

impl From<&str> for TableStyle {
    fn from(s: &str) -> Self {
        match s {
            "modern" => TableStyle::Modern,
            "borderless" => TableStyle::Borderless,
            "ascii" => TableStyle::Ascii,
            "markdown" => TableStyle::Markdown,
            "sharp" => TableStyle::Sharp,
            "psql" => TableStyle::Psql,
            "dots" => TableStyle::Dots,
            _ => TableStyle::default(),
        }
    }
}

impl From<String> for TableStyle {
    fn from(value: String) -> Self {
        TableStyle::from(value.as_str())
    }
}

#[derive(Copy, Clone)]
pub struct TableStyler {
    pub style: TableStyle,
}

impl TableStyler {
    pub fn new(args: &SharedSettings) -> Self {
        if args.non_interactive {
            return Self {
                style: TableStyle::Borderless,
            };
        };

        Self {
            style: args.table_style.unwrap_or_default(),
        }
    }

    pub fn apply(self, table: &mut Table) {
        match self.style {
            TableStyle::Modern => {
                self.apply_modern(table);
            }
            TableStyle::Borderless => {
                self.apply_borderless(table);
            }
            TableStyle::Markdown => {
                self.apply_markdown(table);
            }
            TableStyle::Sharp => {
                self.apply_sharp(table);
            }
            TableStyle::Ascii => {
                self.apply_ascii(table);
            }
            TableStyle::Psql => {
                self.apply_psql(table);
            }
            TableStyle::Dots => {
                self.apply_dots(table);
            }
        }
    }

    fn apply_modern(self, table: &mut Table) -> &Table {
        table.with(Style::modern())
    }

    fn apply_borderless(self, table: &mut Table) -> &Table {
        table.with(Style::empty());
        table.with(tabled::settings::Padding::new(0, 1, 0, 0));
        table.with(Remove::row(Rows::first()));
        table.with(Modify::new(Segment::all()).with(Format::content(|s| s.replace("\n", ","))))
    }

    fn apply_markdown(self, table: &mut Table) -> &Table {
        table.with(Style::markdown())
    }

    fn apply_sharp(self, table: &mut Table) -> &Table {
        table.with(Style::sharp())
    }

    fn apply_psql(self, table: &mut Table) -> &Table {
        table.with(Style::psql())
    }

    fn apply_dots(self, table: &mut Table) -> &Table {
        table.with(Style::dots())
    }

    fn apply_ascii(self, table: &mut Table) -> &Table {
        table.with(Style::ascii())
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
}

impl<'a> ResultHandler<'a> {
    pub fn new(common_args: &'a SharedSettings, command_args: &ArgMatches) -> Self {
        let non_interactive = common_args.non_interactive;
        let quiet = common_args.quiet;
        let idempotently = match command_args.try_get_one::<bool>("idempotently") {
            Ok(val) => val.cloned().unwrap_or(false),
            Err(_) => false,
        };

        let table_styler = TableStyler::new(common_args);

        Self {
            cli_args: common_args,
            table_styler,
            quiet,
            non_interactive,
            idempotently,
            exit_code: None,
        }
    }

    pub fn show_overview(&mut self, result: ClientResult<Overview>) {
        match result {
            Ok(ov) => {
                self.exit_code = Some(ExitCode::Ok);

                let mut table = tables::overview(ov);
                self.table_styler.apply(&mut table);

                println!("{}", table);
            }
            Err(error) => self.report_command_run_error(&error),
        }
    }

    pub fn show_churn(&mut self, result: ClientResult<Overview>) {
        match result {
            Ok(ov) => {
                self.exit_code = Some(ExitCode::Ok);

                let mut table = tables::churn_overview(ov);
                self.table_styler.apply(&mut table);

                println!("{}", table);
            }
            Err(error) => self.report_command_run_error(&error),
        }
    }

    pub fn show_salted_and_hashed_value(&mut self, result: Result<String, HashingError>) {
        match result {
            Ok(value) => {
                self.exit_code = Some(ExitCode::Ok);

                let mut table = tables::show_salted_and_hashed_value(value);
                self.table_styler.apply(&mut table);

                println!("{}", table);
            }
            Err(error) => self.report_hashing_error(&error),
        }
    }

    pub fn tabular_result<T>(&mut self, result: ClientResult<Vec<T>>)
    where
        T: fmt::Debug + Tabled,
    {
        match result {
            Ok(rows) => {
                self.exit_code = Some(ExitCode::Ok);

                let mut table = Table::new(rows);
                self.table_styler.apply(&mut table);

                println!("{}", table);
            }
            Err(error) => self.report_command_run_error(&error),
        }
    }

    pub fn single_value_result<T: fmt::Display>(&mut self, result: ClientResult<T>) {
        match result {
            Ok(output) => {
                self.exit_code = Some(ExitCode::Ok);
                println!("{}", output)
            }
            Err(error) => self.report_command_run_error(&error),
        }
    }

    pub fn memory_breakdown_in_bytes_result(&mut self, result: ClientResult<NodeMemoryBreakdown>) {
        match result {
            Ok(output) => {
                self.exit_code = Some(ExitCode::Ok);

                let mut table = tables::memory_breakdown_in_bytes(output);
                self.table_styler.apply(&mut table);

                println!("{}", table);
            }
            Err(error) => self.report_command_run_error(&error),
        }
    }

    pub fn memory_breakdown_in_percent_result(
        &mut self,
        result: ClientResult<NodeMemoryBreakdown>,
    ) {
        match result {
            Ok(output) => {
                self.exit_code = Some(ExitCode::Ok);

                let mut table = tables::memory_breakdown_in_percent(output);
                self.table_styler.apply(&mut table);

                println!("{}", table);
            }
            Err(error) => self.report_command_run_error(&error),
        }
    }

    pub fn schema_definition_sync_status_result(
        &mut self,
        result: ClientResult<SchemaDefinitionSyncStatus>,
    ) {
        match result {
            Ok(output) => {
                self.exit_code = Some(ExitCode::Ok);

                let mut table = tables::schema_definition_sync_status(output);
                self.table_styler.apply(&mut table);

                println!("{}", table);
            }
            Err(error) => self.report_command_run_error(&error),
        }
    }

    pub fn warm_standby_replication_status_result(
        &mut self,
        result: ClientResult<WarmStandbyReplicationStatus>,
    ) {
        match result {
            Ok(data) => {
                self.exit_code = Some(ExitCode::Ok);

                let tb = Table::builder(data.virtual_hosts);
                let mut table = tb.build();
                table.with(Panel::header("Warm Standby Replication Status"));
                self.table_styler.apply(&mut table);

                println!("{}", table);
            }
            Err(error) => self.report_command_run_error(&error),
        }
    }

    pub fn no_output_on_success<T>(&mut self, result: ClientResult<T>) {
        match result {
            Ok(_) => {
                self.exit_code = Some(ExitCode::Ok);
            }
            Err(error) => self.report_command_run_error(&error),
        }
    }

    pub fn delete_operation_result<T>(&mut self, result: ClientResult<T>) {
        match result {
            Ok(_) => {
                self.exit_code = Some(ExitCode::Ok);
            }
            Err(error) => match error {
                ClientError::ClientErrorResponse {
                    status_code: http_code,
                    ..
                } if http_code == StatusCode::NOT_FOUND => {
                    if self.idempotently {
                        self.exit_code = Some(ExitCode::Ok)
                    } else {
                        self.report_command_run_error(&error)
                    }
                }
                ClientError::NotFound => {
                    if self.idempotently {
                        self.exit_code = Some(ExitCode::Ok)
                    } else {
                        self.report_command_run_error(&error)
                    }
                }
                _ => self.report_command_run_error(&error),
            },
        }
    }

    pub fn health_check_result(&mut self, result: ClientResult<()>) {
        match result {
            Ok(_) => {
                self.exit_code = Some(ExitCode::Ok);
                if !self.quiet {
                    println!("health check passed");
                }
            }
            Err(ClientError::HealthCheckFailed {
                path,
                details,
                status_code,
            }) => {
                self.exit_code = Some(ExitCode::Unavailable);

                let mut table = tables::health_check_failure(&path, status_code, details);
                self.table_styler.apply(&mut table);

                println!("{}", table);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                self.exit_code = Some(ExitCode::Unavailable);
            }
        }
    }

    pub fn report_pre_command_run_error(&mut self, error: &CommandRunError) {
        eprintln!("{}", error);
        let code = match error {
            CommandRunError::UnknownCommandTarget { .. } => ExitCode::Usage,
            CommandRunError::CertificateFileCouldNotBeLoaded1 { .. } => ExitCode::DataErr,
            CommandRunError::CertificateFileCouldNotBeLoaded2 { .. } => ExitCode::DataErr,
            CommandRunError::IoError { .. } => ExitCode::DataErr,
            _ => ExitCode::Usage,
        };
        self.exit_code = Some(code);
    }

    //
    // Implementation
    //

    fn report_command_run_error(&mut self, error: &HttpClientError) {
        let mut table = tables::failure_details(error);
        self.table_styler.apply(&mut table);
        eprintln!("{}", table);
        let code = client_error_to_exit_code(error);
        self.exit_code = Some(code);
    }

    fn report_hashing_error(&mut self, error: &HashingError) {
        let mut table = tables::hashing_error_details(error);
        self.table_styler.apply(&mut table);
        eprintln!("{}", table);
        self.exit_code = Some(ExitCode::DataErr);
    }
}

// We cannot implement From<T> for two types in other crates, so…
pub(crate) fn client_error_to_exit_code(error: &HttpClientError) -> ExitCode {
    match error {
        ClientError::MissingProperty { .. } => ExitCode::DataErr,
        ClientError::UnsupportedArgumentValue { .. } => ExitCode::DataErr,
        ClientError::ClientErrorResponse { .. } => ExitCode::DataErr,
        ClientError::ServerErrorResponse { .. } => ExitCode::Unavailable,
        ClientError::HealthCheckFailed { .. } => ExitCode::Unavailable,
        ClientError::NotFound => ExitCode::DataErr,
        ClientError::MultipleMatchingBindings => ExitCode::DataErr,
        ClientError::InvalidHeaderValue { error: _ } => ExitCode::DataErr,
        ClientError::IncompatibleBody { .. } => ExitCode::DataErr,
        ClientError::RequestError { .. } => ExitCode::IoErr,
        ClientError::Other => ExitCode::Usage,
    }
}
