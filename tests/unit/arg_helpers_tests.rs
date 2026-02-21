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

use clap::{Arg, ArgMatches, Command};
use rabbitmqadmin::arg_helpers::ArgMatchesExt;
use rabbitmqadmin::errors::CommandRunError;

type CommandResult<T> = Result<T, CommandRunError>;

fn make_matches(args: &[&str]) -> ArgMatches {
    Command::new("test")
        .arg(Arg::new("name").long("name"))
        .arg(Arg::new("count").long("count"))
        .arg(
            Arg::new("flag")
                .long("flag")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches_from(args)
}

#[test]
fn test_required_string_present() {
    let matches = make_matches(&["test", "--name", "hello"]);
    let result = matches.required_string("name");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn test_required_string_missing() {
    let matches = make_matches(&["test"]);
    let result = matches.required_string("name");
    assert!(result.is_err());
    match result.unwrap_err() {
        CommandRunError::MissingRequiredArgument { name } => {
            assert_eq!(name, "name");
        }
        _ => panic!("Expected MissingRequiredArgument"),
    }
}

#[test]
fn test_optional_string_present() {
    let matches = make_matches(&["test", "--name", "world"]);
    assert_eq!(matches.optional_string("name"), Some("world".to_string()));
}

#[test]
fn test_optional_string_missing() {
    let matches = make_matches(&["test"]);
    assert_eq!(matches.optional_string("name"), None);
}

#[test]
fn test_parse_required_valid() {
    let matches = make_matches(&["test", "--count", "42"]);
    let result: CommandResult<i32> = matches.parse_required("count");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_parse_required_invalid() {
    let matches = make_matches(&["test", "--count", "not_a_number"]);
    let result: CommandResult<i32> = matches.parse_required("count");
    assert!(result.is_err());
    match result.unwrap_err() {
        CommandRunError::InvalidArgumentValue { name, message: _ } => {
            assert_eq!(name, "count");
        }
        _ => panic!("Expected InvalidArgumentValue"),
    }
}

#[test]
fn test_parse_required_missing() {
    let matches = make_matches(&["test"]);
    let result: CommandResult<i32> = matches.parse_required("count");
    assert!(result.is_err());
    match result.unwrap_err() {
        CommandRunError::MissingRequiredArgument { name } => {
            assert_eq!(name, "count");
        }
        _ => panic!("Expected MissingRequiredArgument"),
    }
}
