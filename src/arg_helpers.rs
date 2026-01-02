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

use crate::errors::CommandRunError;
use clap::ArgMatches;
use std::fmt::Display;
use std::str::FromStr;

type CommandResult<T> = Result<T, CommandRunError>;

pub trait ArgMatchesExt {
    fn str_arg(&self, name: &str) -> &String;
    fn string_arg(&self, name: &str) -> String;

    fn required_string(&self, name: &str) -> CommandResult<String>;
    fn optional_string(&self, name: &str) -> Option<String>;
    fn optional_typed<T: Clone + Send + Sync + 'static>(&self, name: &str) -> Option<T>;
    fn optional_typed_or<T: Clone + Send + Sync + 'static>(&self, name: &str, default: T) -> T;
    fn parse_required<T: FromStr>(&self, name: &str) -> CommandResult<T>
    where
        T::Err: Display;
}

impl ArgMatchesExt for ArgMatches {
    fn str_arg(&self, name: &str) -> &String {
        self.get_one::<String>(name).unwrap()
    }

    fn string_arg(&self, name: &str) -> String {
        self.get_one::<String>(name).cloned().unwrap()
    }

    fn required_string(&self, name: &str) -> CommandResult<String> {
        self.get_one::<String>(name).cloned().ok_or_else(|| {
            CommandRunError::MissingRequiredArgument {
                name: name.to_string(),
            }
        })
    }

    fn optional_string(&self, name: &str) -> Option<String> {
        self.get_one::<String>(name).cloned()
    }

    fn optional_typed<T: Clone + Send + Sync + 'static>(&self, name: &str) -> Option<T> {
        self.get_one::<T>(name).cloned()
    }

    fn optional_typed_or<T: Clone + Send + Sync + 'static>(&self, name: &str, default: T) -> T {
        self.get_one::<T>(name).cloned().unwrap_or(default)
    }

    fn parse_required<T: FromStr>(&self, name: &str) -> CommandResult<T>
    where
        T::Err: Display,
    {
        let value = self.required_string(name)?;
        value
            .parse::<T>()
            .map_err(|e| CommandRunError::InvalidArgumentValue {
                name: name.to_string(),
                message: e.to_string(),
            })
    }
}
