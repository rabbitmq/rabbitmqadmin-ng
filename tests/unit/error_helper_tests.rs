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
use rabbitmq_http_client::error::Error as ApiClientError;
use reqwest::StatusCode;

fn create_http_client_error(status: StatusCode) -> HttpClientError {
    ApiClientError::ClientErrorResponse {
        url: None,
        status_code: status,
        body: None,
        error_details: None,
        headers: None,
        backtrace: Backtrace::new(),
    }
}

#[test]
fn test_http_client_error_is_not_found() {
    let err = create_http_client_error(StatusCode::NOT_FOUND);
    assert!(err.is_not_found());
}

#[test]
fn test_http_client_error_is_not_found_variant() {
    let err: HttpClientError = ApiClientError::NotFound;
    assert!(err.is_not_found());
}

#[test]
fn test_http_client_error_is_not_found_false() {
    let err = create_http_client_error(StatusCode::BAD_REQUEST);
    assert!(!err.is_not_found());
}

#[test]
fn test_http_client_error_is_unauthorized_401() {
    let err = create_http_client_error(StatusCode::UNAUTHORIZED);
    assert!(err.is_unauthorized());
}

#[test]
fn test_http_client_error_is_forbidden_403() {
    let err = create_http_client_error(StatusCode::FORBIDDEN);
    assert!(err.is_forbidden());
}

#[test]
fn test_http_client_error_is_forbidden_false() {
    let err = create_http_client_error(StatusCode::BAD_REQUEST);
    assert!(!err.is_forbidden());
}

#[test]
fn test_http_client_error_is_unauthorized_false() {
    let err = create_http_client_error(StatusCode::BAD_REQUEST);
    assert!(!err.is_unauthorized());
}

#[test]
fn test_http_client_error_is_already_exists() {
    let err = create_http_client_error(StatusCode::CONFLICT);
    assert!(err.is_already_exists());
}

#[test]
fn test_http_client_error_status_code() {
    let err = create_http_client_error(StatusCode::BAD_REQUEST);
    assert_eq!(err.status_code(), Some(StatusCode::BAD_REQUEST));
}

#[test]
fn test_http_client_error_status_code_not_found() {
    let err: HttpClientError = ApiClientError::NotFound;
    assert_eq!(err.status_code(), Some(StatusCode::NOT_FOUND));
}
