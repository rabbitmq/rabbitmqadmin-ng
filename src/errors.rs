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

use rabbitmq_http_client::error::{ConversionError, Error as ApiClientError};
use rabbitmq_http_client::{blocking_api::HttpClientError, responses::HealthCheckFailureDetails};
use reqwest::{
    StatusCode,
    header::{HeaderMap, InvalidHeaderValue},
};
use url::Url;

#[derive(thiserror::Error, Debug)]
pub enum CommandRunError {
    #[error("Asked to run an unknown command '{command} {subcommand}")]
    UnknownCommandTarget { command: String, subcommand: String },
    #[error(
        "Local TLS certificate file at {local_path} does not exist, cannot be read or passed as a PEM file: {cause}"
    )]
    CertificateFileCouldNotBeLoaded1 {
        local_path: String,
        cause: reqwest::Error,
    },
    #[error(
        "Local TLS certificate file at {local_path} does not exist, cannot be read or passed as a PEM file: {cause}"
    )]
    CertificateFileCouldNotBeLoaded2 {
        local_path: String,
        cause: rustls::pki_types::pem::Error,
    },
    #[error("Run into an I/O error when loading a file: {0}")]
    IoError(std::io::Error),
    #[error(
        "Local TLS certificate file at {local_path} does not exist, cannot be read or passed as a PEM file: {cause}"
    )]
    CertificateStoreRejectedCertificate {
        local_path: String,
        cause: rustls::Error,
    },
    #[error("TLS certificate file at {local_path} does not exist or is not readable")]
    CertificateFileNotFound { local_path: String },
    #[error(
        "TLS certificate file at {local_path} could not be parsed, is empty or contains no valid certificates"
    )]
    CertificateFileEmpty { local_path: String },
    #[error("TLS certificate file at {local_path} contains invalid PEM data: {details}")]
    CertificateFileInvalidPem { local_path: String, details: String },
    #[error("TLS private key file at {local_path} contains an unsupported key type or format")]
    PrivateKeyFileUnsupported { local_path: String },
    #[error("TLS certificate and private key files do not match")]
    CertificateKeyMismatch { cert_path: String, key_path: String },
    #[error("API responded with a client error: status code of {status_code}")]
    ClientError {
        status_code: StatusCode,
        url: Option<Url>,
        body: Option<String>,
        headers: Option<HeaderMap>,
    },
    #[error("API responded with a client error: status code of {status_code}")]
    ServerError {
        status_code: StatusCode,
        url: Option<Url>,
        body: Option<String>,
        headers: Option<HeaderMap>,
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
    #[error("{message}")]
    MissingOptions { message: String },
    #[error("Missing argument value for property (field) {property}")]
    MissingArgumentValue { property: String },
    #[error("Unsupported argument value for property (field) {property}")]
    UnsupportedArgumentValue { property: String },
    #[error("This request produces an invalid HTTP header value")]
    InvalidHeaderValue { error: InvalidHeaderValue },
    #[error("Response is incompatible with the target data type")]
    IncompatibleBody { error: ConversionError },
    #[error("encountered an error when performing an HTTP request")]
    RequestError { error: reqwest::Error },
    #[error("an unspecified error")]
    Other,
}

impl From<std::io::Error> for CommandRunError {
    fn from(value: std::io::Error) -> Self {
        CommandRunError::IoError(value)
    }
}

impl From<HttpClientError> for CommandRunError {
    fn from(value: HttpClientError) -> Self {
        use ApiClientError::*;
        match value {
            UnsupportedArgumentValue { property } => Self::UnsupportedArgumentValue { property },
            ClientErrorResponse { status_code, url, body, headers, .. } => {
                Self::ClientError { status_code, url, body, headers }
            }
            ServerErrorResponse { status_code, url, body, headers, .. } => {
                Self::ServerError { status_code, url, body, headers }
            }
            HealthCheckFailed { path, details, status_code } => {
                Self::HealthCheckFailed { health_check_path: path, details, status_code }
            }
            NotFound => Self::NotFound,
            MultipleMatchingBindings => Self::ConflictingOptions {
                message: "multiple bindings match, cannot determine which binding to delete without explicitly provided binding properties".to_owned()
            },
            InvalidHeaderValue { error } => Self::InvalidHeaderValue { error },
            RequestError { error, .. } => Self::RequestError { error },
            Other => Self::Other,
            MissingProperty { argument } => Self::MissingArgumentValue { property: argument },
            IncompatibleBody { error, .. } => Self::IncompatibleBody { error },
        }
    }
}
