use crate::errors::CommandRunError;
// Copyright (C) 2023-2024 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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
use clap::ArgMatches;
use rabbitmq_http_client::blocking_api::{HttpClientError, Result as ClientResult};
use rabbitmq_http_client::error::Error as ClientError;
use rabbitmq_http_client::responses::Overview;
use crate::tables;
use std::fmt;
use sysexits::ExitCode;
use tabled::settings::object::Rows;
use tabled::settings::{Remove, Style};
use tabled::{Table, Tabled};

#[derive(Copy, Clone)]
pub struct TableStyler {
    non_interactive: bool,
}

impl TableStyler {
    pub fn new(args: &ArgMatches) -> Self {
        let non_interactive = args
            .get_one::<bool>("non_interactive")
            .cloned()
            .unwrap_or(false);

        Self { non_interactive }
    }

    pub fn apply(self, table: &mut Table) {
        if self.non_interactive {
            table.with(Style::empty());
            table.with(Remove::row(Rows::first()));
        } else {
            table.with(Style::modern());
        }
    }
}

#[allow(dead_code)]
pub struct ResultHandler {
    pub non_interactive: bool,
    pub quiet: bool,
    pub exit_code: Option<ExitCode>,
    table_styler: TableStyler,
}

impl ResultHandler {
    pub fn new(args: &ArgMatches) -> Self {
        let non_interactive = args
            .get_one::<bool>("non_interactive")
            .cloned()
            .unwrap_or(false);
        let quiet = args.get_one::<bool>("quiet").cloned().unwrap_or(false);
        let table_formatter = TableStyler::new(args);

        Self {
            quiet,
            non_interactive,
            exit_code: None,
            table_styler: table_formatter,
        }
    }

    pub fn show_overview(&mut self, result: ClientResult<Overview>) {
        match result {
            Ok(ov) => {
                self.exit_code = Some(ExitCode::Ok);

                let mut table = tables::overview_table(ov);
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

                let mut table = tables::churn_overview_table(ov);
                self.table_styler.apply(&mut table);

                println!("{}", table);
            }
            Err(error) => self.report_command_run_error(&error),
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

    pub fn no_output_on_success<T>(&mut self, result: Result<T, HttpClientError>) {
        match result {
            Ok(_) => {
                self.exit_code = Some(ExitCode::Ok);
            }
            Err(error) => self.report_command_run_error(&error),
        }
    }

    pub fn report_pre_command_run_error(&mut self, error: &CommandRunError) {
        eprintln!("{}", error);
        self.exit_code = Some(ExitCode::Usage);
    }

    //
    // Implementation
    //

    fn report_command_run_error(&mut self, error: &HttpClientError) {
        eprintln!("{}", error);
        let code = client_error_to_exit_code(error);
        self.exit_code = Some(code);
    }
}

// We cannot implement From<T> for two types in other crates, so…
pub(crate) fn client_error_to_exit_code(error: &HttpClientError) -> ExitCode {
    match error {
        ClientError::ClientErrorResponse { .. } => ExitCode::DataErr,
        ClientError::ServerErrorResponse { .. } => ExitCode::Unavailable,
        ClientError::HealthCheckFailed { .. } => ExitCode::Unavailable,
        ClientError::NotFound => ExitCode::DataErr,
        ClientError::MultipleMatchingBindings => ExitCode::DataErr,
        ClientError::InvalidHeaderValue { error: _ } => ExitCode::DataErr,
        ClientError::RequestError { .. } => ExitCode::IoErr,
        ClientError::Other => ExitCode::Usage,
    }
}
