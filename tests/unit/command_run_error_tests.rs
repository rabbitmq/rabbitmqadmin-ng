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

use rabbitmq_http_client::error::ConversionError;
use rabbitmqadmin::errors::CommandRunError;
use reqwest::header::HeaderValue;

#[test]
fn test_request_error_includes_underlying_error() {
    // Build a reqwest error by attempting a request to an invalid URL
    let err = reqwest::blocking::get("not-a-valid-url").unwrap_err();
    let cmd_err = CommandRunError::RequestError { error: err };
    let msg = cmd_err.to_string();
    assert!(
        msg.starts_with("Encountered an error when performing an HTTP request:"),
        "unexpected message: {}",
        msg
    );
    // The underlying reqwest error detail must be present beyond the static prefix
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
