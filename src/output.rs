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
use rabbitmq_http_client::blocking_api::Result as ClientResult;
use rabbitmq_http_client::responses::Overview;
use std::{fmt, process};
use tabled::settings::object::Rows;
use tabled::settings::{Remove, Style};
use tabled::{Table, Tabled};

pub fn print_overview_or_fail(result: ClientResult<Overview>, args: &ArgMatches) {
    let non_interactive = *(args.get_one::<bool>("non_interactive").unwrap());

    match result {
        Ok(ov) => {
            let mut table = format::overview_table(ov);

            if non_interactive {
                table.with(Style::empty());
                table.with(Remove::row(Rows::first()));
            } else {
                table.with(Style::modern());
            }
            println!("{}", table);
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1)
        }
    }
}

pub fn print_churn_overview_or_fail(result: ClientResult<Overview>, args: &ArgMatches) {
    let non_interactive = *(args.get_one::<bool>("non_interactive").unwrap());

    match result {
        Ok(ov) => {
            let mut table = format::churn_overview_table(ov);

            if non_interactive {
                table.with(Style::empty());
                table.with(Remove::row(Rows::first()));
            } else {
                table.with(Style::modern());
            }
            println!("{}", table);
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1)
        }
    }
}

pub fn print_table_or_fail<T>(result: ClientResult<Vec<T>>, args: &ArgMatches)
where
    T: fmt::Debug + Tabled,
{
    let non_interactive = *(args.get_one::<bool>("non_interactive").unwrap());

    match result {
        Ok(rows) => {
            let mut table = Table::new(rows);

            if non_interactive {
                table.with(Style::empty());
                table.with(Remove::row(Rows::first()));
            } else {
                table.with(Style::modern());
            }

            println!("{}", table);
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1)
        }
    }
}
