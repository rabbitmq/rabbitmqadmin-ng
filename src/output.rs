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
use crate::format;
use clap::ArgMatches;
use rabbitmq_http_client::blocking_api::{HttpClientError, Result as ClientResult};
use rabbitmq_http_client::error::Error as ClientError;
use rabbitmq_http_client::responses::Overview;
use std::{fmt, process};
use sysexits::ExitCode;
use tabled::settings::object::Rows;
use tabled::settings::{Remove, Style};
use tabled::{Table, Tabled};

#[allow(dead_code)]
pub struct ResultHandler {
    pub non_interactive: bool,
    pub quiet: bool,
    pub exit_code: Option<ExitCode>,
}

impl ResultHandler {
    pub fn new(args: &ArgMatches) -> Self {
        let non_interactive = args
            .get_one::<bool>("non_interactive")
            .cloned()
            .unwrap_or(false);
        let quiet = args.get_one::<bool>("quiet").cloned().unwrap_or(false);

        Self {
            quiet,
            non_interactive,
            exit_code: None,
        }
    }

    pub fn show_overview(self: &mut Self, result: ClientResult<Overview>) {
        match result {
            Ok(ov) => {
                self.exit_code = Some(ExitCode::Ok);

                let mut table = format::overview_table(ov);

                if self.non_interactive {
                    table.with(Style::empty());
                    table.with(Remove::row(Rows::first()));
                } else {
                    table.with(Style::modern());
                }

                self.exit_code = Some(ExitCode::Ok);
                println!("{}", table);
            }
            Err(error) => self.print_to_stderr_and_exit(&error),
        }
    }

    pub fn show_churn(self: &mut Self, result: ClientResult<Overview>) {
        match result {
            Ok(ov) => {
                self.exit_code = Some(ExitCode::Ok);

                let mut table = format::churn_overview_table(ov);

                if self.non_interactive {
                    table.with(Style::empty());
                    table.with(Remove::row(Rows::first()));
                } else {
                    table.with(Style::modern());
                }
                println!("{}", table);
            }
            Err(error) => self.print_to_stderr_and_exit(&error),
        }
    }

    pub fn tabular_result<T>(self: &mut Self, result: ClientResult<Vec<T>>)
    where
        T: fmt::Debug + Tabled,
    {
        match result {
            Ok(rows) => {
                self.exit_code = Some(ExitCode::Ok);

                let mut table = Table::new(rows);

                if self.non_interactive {
                    table.with(Style::empty());
                    table.with(Remove::row(Rows::first()));
                } else {
                    table.with(Style::modern());
                }

                println!("{}", table);
            }
            Err(error) => self.print_to_stderr_and_exit(&error),
        }
    }

    pub fn single_value_result<T: fmt::Display>(self: &mut Self, result: ClientResult<T>) {
        match result {
            Ok(output) => {
                self.exit_code = Some(ExitCode::Ok);
                println!("{}", output)
            }
            Err(error) => self.print_to_stderr_and_exit(&error),
        }
    }

    pub fn no_output_on_success<T>(self: &mut Self, result: ClientResult<T>) {
        match result {
            Ok(_) => {
                self.exit_code = Some(ExitCode::Ok);
                process::exit(ExitCode::Ok.into())
            }
            Err(error) => self.print_to_stderr_and_exit(&error),
        }
    }

    //
    // Implementation
    //

    fn print_to_stderr_and_exit(self: &mut Self, error: &HttpClientError) {
        eprintln!("{}", error);
        let code = client_error_to_exit_code(&error);
        self.exit_code = Some(code);
        process::exit(code.into())
    }
}

// We cannot implement From<T> for two types in other crates, so…
pub(crate) fn client_error_to_exit_code(error: &HttpClientError) -> ExitCode {
    match error {
        ClientError::ClientErrorResponse {
            status_code: _,
            response: _,
            backtrace: _,
        } => ExitCode::DataErr,
        ClientError::ServerErrorResponse {
            status_code: _,
            response: _,
            backtrace: _,
        } => ExitCode::Unavailable,
        ClientError::HealthCheckFailed {
            details: _,
            status_code: _,
        } => ExitCode::Unavailable,
        ClientError::NotFound => ExitCode::DataErr,
        ClientError::MultipleMatchingBindings => ExitCode::DataErr,
        ClientError::InvalidHeaderValue { error: _ } => ExitCode::DataErr,
        ClientError::RequestError {
            error: _,
            backtrace: _,
        } => ExitCode::IoErr,
        ClientError::Other => ExitCode::Usage,
    }
}
