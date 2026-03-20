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

use backtrace::Backtrace;
use rabbitmq_http_client::blocking_api::HttpClientError;
use rabbitmq_http_client::error::ConversionError;
use rabbitmq_http_client::error::Error as ApiClientError;
use rabbitmqadmin::config::SharedSettings;
use rabbitmqadmin::errors::CommandRunError;
use rabbitmqadmin::output::ResultHandler;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use sysexits::ExitCode;
fn make_handler<'a>(settings: &'a SharedSettings) -> ResultHandler<'a> {
    let matches = clap::Command::new("test").get_matches_from(["test"]);
    ResultHandler::new(settings, &matches)
}

#[test]
fn test_certificate_file_could_not_be_loaded1_message_indicates_parse_failure() {
    let cause = reqwest::blocking::get("not-a-valid-url").unwrap_err();
    let cmd_err = CommandRunError::CertificateFileCouldNotBeLoaded1 {
        local_path: "/path/to/cert.pem".to_owned(),
        cause,
    };
    let msg = cmd_err.to_string();
    assert!(
        msg.contains("/path/to/cert.pem"),
        "path missing from message: {}",
        msg
    );
    assert!(
        msg.contains("parsed"),
        "message should indicate a parse failure: {}",
        msg
    );
}

#[test]
fn test_certificate_file_could_not_be_loaded2_message_indicates_read_failure() {
    let cause = rustls::pki_types::pem::Error::Io(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "permission denied",
    ));
    let cmd_err = CommandRunError::CertificateFileCouldNotBeLoaded2 {
        local_path: "/path/to/key.pem".to_owned(),
        cause,
    };
    let msg = cmd_err.to_string();
    assert!(
        msg.contains("/path/to/key.pem"),
        "path missing from message: {}",
        msg
    );
    assert!(
        msg.contains("read"),
        "message should indicate a read failure: {}",
        msg
    );
}

#[test]
fn test_unknown_command_target_message_has_balanced_quotes() {
    let cmd_err = CommandRunError::UnknownCommandTarget {
        command: "vhosts".to_owned(),
        subcommand: "frobnicate".to_owned(),
    };
    let msg = cmd_err.to_string();
    assert!(
        msg.contains("'vhosts frobnicate'"),
        "expected quoted command in message: {}",
        msg
    );
}

#[test]
fn test_request_error_includes_underlying_error() {
    let err = reqwest::blocking::get("not-a-valid-url").unwrap_err();
    let cmd_err = CommandRunError::RequestError { error: err };
    let msg = cmd_err.to_string();
    assert!(
        msg.starts_with("Encountered an error when performing an HTTP request:"),
        "unexpected message: {}",
        msg
    );
    assert!(
        msg.len() > "Encountered an error when performing an HTTP request:".len(),
        "underlying error detail missing: {}",
        msg
    );
}

#[test]
fn test_invalid_header_value_includes_underlying_error() {
    // Control characters (< 32, except \t) are rejected by HeaderValue
    let err = HeaderValue::from_bytes(&[0x01]).unwrap_err();
    let cmd_err = CommandRunError::InvalidHeaderValue { error: err };
    let msg = cmd_err.to_string();
    assert!(
        msg.starts_with("This request produces an invalid HTTP header value:"),
        "unexpected message: {}",
        msg
    );
    assert!(
        msg.len() > "This request produces an invalid HTTP header value:".len(),
        "underlying error detail missing: {}",
        msg
    );
}

#[test]
fn test_incompatible_body_includes_underlying_error() {
    let err = ConversionError::ParsingError {
        message: "unexpected token".to_owned(),
    };
    let cmd_err = CommandRunError::IncompatibleBody { error: err };
    let msg = cmd_err.to_string();
    assert!(
        msg.contains("unexpected token"),
        "underlying error detail missing: {}",
        msg
    );
}

#[test]
fn test_certificate_key_mismatch_includes_paths() {
    let cmd_err = CommandRunError::CertificateKeyMismatch {
        cert_path: "/path/to/cert.pem".to_owned(),
        key_path: "/path/to/key.pem".to_owned(),
    };
    let msg = cmd_err.to_string();
    assert!(
        msg.contains("/path/to/cert.pem"),
        "cert path missing from message: {}",
        msg
    );
    assert!(
        msg.contains("/path/to/key.pem"),
        "key path missing from message: {}",
        msg
    );
}

#[test]
fn test_request_error_exit_code_is_data_err() {
    let err = reqwest::blocking::get("not-a-valid-url").unwrap_err();
    let cmd_err = CommandRunError::RequestError { error: err };
    let settings = SharedSettings::default();
    let mut handler = make_handler(&settings);
    handler.report_pre_command_run_error(&cmd_err);
    assert_eq!(handler.exit_code, Some(ExitCode::DataErr));
}

#[test]
fn test_client_error_exit_code_is_data_err() {
    let api_err = ApiClientError::ClientErrorResponse {
        status_code: StatusCode::BAD_REQUEST,
        url: None,
        body: None,
        error_details: None,
        headers: None,
        backtrace: Backtrace::new(),
    };
    let cmd_err = CommandRunError::from(HttpClientError::from(api_err));
    let settings = SharedSettings::default();
    let mut handler = make_handler(&settings);
    handler.report_pre_command_run_error(&cmd_err);
    assert_eq!(handler.exit_code, Some(ExitCode::DataErr));
}

#[test]
fn test_not_found_exit_code_is_data_err() {
    let cmd_err = CommandRunError::from(HttpClientError::from(ApiClientError::NotFound));
    let settings = SharedSettings::default();
    let mut handler = make_handler(&settings);
    handler.report_pre_command_run_error(&cmd_err);
    assert_eq!(handler.exit_code, Some(ExitCode::DataErr));
}
