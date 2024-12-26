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
use rabbitmq_http_client::blocking_api::HttpClientError;
use rabbitmq_http_client::responses::{HealthCheckFailureDetails, Overview};
use reqwest::StatusCode;
use tabled::settings::Panel;
use tabled::{Table, Tabled};
use url::Url;

#[derive(Tabled)]
struct OverviewRow<'a> {
    key: &'a str,
    value: String,
}

#[derive(Tabled)]
struct RowOfTwoStrings<'a> {
    key: &'a str,
    value: &'a str,
}

pub fn overview(ov: Overview) -> Table {
    let data = vec![
        OverviewRow {
            key: "Product name",
            value: ov.product_name,
        },
        OverviewRow {
            key: "Product version",
            value: ov.product_version,
        },
        OverviewRow {
            key: "RabbitMQ version",
            value: ov.rabbitmq_version,
        },
        OverviewRow {
            key: "Erlang version",
            value: ov.erlang_version,
        },
        OverviewRow {
            key: "Erlang details",
            value: ov.erlang_full_version,
        },
    ];
    // TODO: if any tags are non-empty, add them to the table
    let tb = Table::builder(data);
    let mut t = tb.build();
    t.with(Panel::header("Overview"));
    t
}

pub fn churn_overview(ov: Overview) -> Table {
    let data = vec![
        OverviewRow {
            key: "Client connections opened",
            value: ov.churn_rates.connection_created.to_string(),
        },
        OverviewRow {
            key: "Client connections closed",
            value: ov.churn_rates.connection_closed.to_string(),
        },
        OverviewRow {
            key: "Client channels opened",
            value: ov.churn_rates.channel_created.to_string(),
        },
        OverviewRow {
            key: "Client channels closed",
            value: ov.churn_rates.channel_closed.to_string(),
        },
        OverviewRow {
            key: "Queues and streams (re)declarations",
            value: ov.churn_rates.queue_declared.to_string(),
        },
        OverviewRow {
            key: "Queues and streams created",
            value: ov.churn_rates.queue_created.to_string(),
        },
        OverviewRow {
            key: "Queues and streams deleted",
            value: ov.churn_rates.queue_deleted.to_string(),
        },
    ];
    let tb = Table::builder(data);
    let mut t = tb.build();
    t.with(Panel::header(
        "Entity (connections, queues, etc) churn over the most recent sampling period",
    ));
    t
}

pub fn failure_details(error: &HttpClientError) -> Table {
    match error {
        HttpClientError::ClientErrorResponse {
            status_code,
            url,
            body,
            ..
        } => generic_failed_request_details(status_code, url, body),
        HttpClientError::ServerErrorResponse {
            status_code,
            url,
            body,
            ..
        } => generic_failed_request_details(status_code, url, body),
        HttpClientError::HealthCheckFailed {
            status_code,
            path,
            details,
        } => {
            let path_s = path.clone();
            let status_code_s = format!("{}", status_code);
            let mut data = vec![
                RowOfTwoStrings {
                    key: "result",
                    value: "request failed",
                },
                RowOfTwoStrings {
                    key: "status_code",
                    value: status_code_s.as_str(),
                },
                RowOfTwoStrings {
                    key: "path",
                    value: path_s.as_str(),
                },
            ];

            match details {
                HealthCheckFailureDetails::AlarmCheck(details) => {
                    data.push(RowOfTwoStrings {
                        key: "reason",
                        value: details.reason.as_str(),
                    });
                }
                HealthCheckFailureDetails::NodeIsQuorumCritical(details) => {
                    data.push(RowOfTwoStrings {
                        key: "reason",
                        value: details.reason.as_str(),
                    });
                }
            };

            let tb = Table::builder(data);
            tb.build()
        }
        HttpClientError::NotFound => {
            let status_code_s = format!("{}", StatusCode::NOT_FOUND);
            let data = vec![
                RowOfTwoStrings {
                    key: "result",
                    value: "request failed",
                },
                RowOfTwoStrings {
                    key: "status_code",
                    value: status_code_s.as_str(),
                },
            ];

            let tb = Table::builder(data);
            tb.build()
        }
        HttpClientError::MultipleMatchingBindings => {
            let data = vec![
                RowOfTwoStrings {
                    key: "result",
                    value: "request failed",
                },
                RowOfTwoStrings {
                    key: "status_code",
                    value: StatusCode::CONFLICT.as_str(),
                },
                RowOfTwoStrings {
                    key: "reason",
                    value: "multiple bindings found between the source and destination, please specify a --routing-key of the target binding"
                }
            ];

            let tb = Table::builder(data);
            tb.build()
        }
        HttpClientError::InvalidHeaderValue { .. } => {
            let reason = "invalid HTTP request header value";
            let data = vec![
                RowOfTwoStrings {
                    key: "result",
                    value: "request failed",
                },
                RowOfTwoStrings {
                    key: "reason",
                    value: reason,
                },
            ];

            let tb = Table::builder(data);
            tb.build()
        }
        HttpClientError::RequestError {
            error,
            backtrace: _,
        } => {
            let reason = format!("HTTP API request failed: {}", error);
            let data = vec![
                RowOfTwoStrings {
                    key: "result",
                    value: "request failed",
                },
                RowOfTwoStrings {
                    key: "reason",
                    value: reason.as_str(),
                },
            ];

            let tb = Table::builder(data);
            tb.build()
        }
        HttpClientError::Other => {
            let data = vec![
                RowOfTwoStrings {
                    key: "result",
                    value: "request failed",
                },
                RowOfTwoStrings {
                    key: "reason",
                    value: "(not available)",
                },
            ];

            let tb = Table::builder(data);
            tb.build()
        }
    }
}

fn generic_failed_request_details(
    status_code: &StatusCode,
    url: &Option<Url>,
    body: &Option<String>,
) -> Table {
    let status_code_s = status_code.to_string();
    let url_s = url.clone().unwrap().to_string();
    let body_s = body.clone().unwrap_or("N/A".to_string());

    let data = vec![
        RowOfTwoStrings {
            key: "result",
            value: "request failed",
        },
        RowOfTwoStrings {
            key: "status_code",
            value: status_code_s.as_str(),
        },
        RowOfTwoStrings {
            key: "url",
            value: url_s.as_str(),
        },
        RowOfTwoStrings {
            key: "body",
            value: body_s.as_str(),
        },
    ];

    let tb = Table::builder(data);
    tb.build()
}

pub fn health_check_failure(
    path: &str,
    status_code: StatusCode,
    details: HealthCheckFailureDetails,
) -> Table {
    let reason = match details {
        HealthCheckFailureDetails::AlarmCheck(ref details) => details.reason.clone(),
        HealthCheckFailureDetails::NodeIsQuorumCritical(ref details) => details.reason.clone(),
    };
    let code_str = format!("{}", status_code);

    let vec = vec![
        RowOfTwoStrings {
            key: "result",
            value: "health check failed",
        },
        RowOfTwoStrings {
            key: "path",
            value: path,
        },
        RowOfTwoStrings {
            key: "status_code",
            value: code_str.as_str(),
        },
        RowOfTwoStrings {
            key: "reason",
            value: reason.as_str(),
        },
    ];
    let mut tb = Table::builder(vec);
    match details {
        HealthCheckFailureDetails::AlarmCheck(
            rabbitmq_http_client::responses::ClusterAlarmCheckDetails { reason: _, alarms },
        ) => {
            for alarm in alarms {
                let key = format!("alarm in effect on node {}", alarm.node);
                let value = alarm.resource;
                tb.push_record([key.as_str(), value.as_str()]);
            }
        }
        HealthCheckFailureDetails::NodeIsQuorumCritical(
            rabbitmq_http_client::responses::QuorumCriticalityCheckDetails { reason: _, queues },
        ) => {
            for q in queues {
                let key = "affected queue, stream or internal component";
                let value = q.readable_name;
                tb.push_record([key, value.as_str()]);
            }
        }
    };

    tb.build()
}
