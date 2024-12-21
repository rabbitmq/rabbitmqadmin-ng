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

use rabbitmq_http_client::error::Error as ApiClientError;
use rabbitmq_http_client::{blocking_api::HttpClientError, responses::HealthCheckFailureDetails};
use reqwest::blocking::Response;
use reqwest::{header::InvalidHeaderValue, StatusCode};

#[derive(thiserror::Error, Debug)]
pub enum CommandRunError {
    #[error("Asked to run an unknown command '{command} {subcommand}")]
    UnknownCommandTarget { command: String, subcommand: String },
    #[error("API responded with a client error: status code of {status_code}")]
    ClientError {
        status_code: StatusCode,
        response: Option<Response>,
    },
    #[error("API responded with a client error: status code of {status_code}")]
    ServerError {
        status_code: StatusCode,
        response: Option<Response>,
    },
    #[error("Health check failed")]
    HealthCheckFailed {
        health_check_path: String,
        details: HealthCheckFailureDetails,
        status_code: StatusCode,
    },
    #[error("API responded with a 404 Not Found")]
    NotFound,
    #[error("{message}")]
    ConflictingOptions { message: String },
    #[error("This request produces an invalid HTTP header value")]
    InvalidHeaderValue { error: InvalidHeaderValue },
    #[error("encountered an error when performing an HTTP request")]
    RequestError { error: reqwest::Error },
    #[error("an unspecified error")]
    Other,
}

impl From<HttpClientError> for CommandRunError {
    fn from(value: HttpClientError) -> Self {
        match value {
            ApiClientError::ClientErrorResponse { status_code, response, .. } => {
                Self::ClientError { status_code, response }
            },
            ApiClientError::ServerErrorResponse { status_code, response, .. } => {
                Self::ServerError { status_code, response }
            },
            ApiClientError::HealthCheckFailed { path, details, status_code } => {
                Self::HealthCheckFailed { health_check_path: path, details, status_code }
            },
            ApiClientError::NotFound => Self::NotFound,
            ApiClientError::MultipleMatchingBindings => Self::ConflictingOptions {
                message: "multiple bindings match, cannot determing which binding to delete without explicitly provided binding properties".to_owned()
            },
            ApiClientError::InvalidHeaderValue { error } => {
                Self::InvalidHeaderValue { error }
            },
            ApiClientError::RequestError { error, .. } => Self::RequestError { error },
            ApiClientError::Other => Self::Other,
        }
    }
}
