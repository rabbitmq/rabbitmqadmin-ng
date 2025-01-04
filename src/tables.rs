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
use rabbitmq_http_client::blocking_api::HttpClientError;
use rabbitmq_http_client::responses::{
    ClusterAlarmCheckDetails, HealthCheckFailureDetails, NodeMemoryBreakdown, Overview,
    QuorumCriticalityCheckDetails,
};
use reqwest::StatusCode;
use tabled::settings::Panel;
use tabled::{Table, Tabled};
use url::Url;

#[derive(Debug, Tabled)]
struct OverviewRow<'a> {
    key: &'a str,
    value: String,
}

#[derive(Debug, Tabled)]
struct RowOfTwoStrings<'a, T: ?Sized + std::fmt::Display> {
    key: &'a str,
    value: &'a T,
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

            let reason = match details {
                HealthCheckFailureDetails::AlarmCheck(details) => details.reason.clone(),
                HealthCheckFailureDetails::NodeIsQuorumCritical(details) => details.reason.clone(),
                HealthCheckFailureDetails::NoActivePortListener(details) => details.reason.clone(),
                HealthCheckFailureDetails::NoActiveProtocolListener(details) => {
                    details.reason.clone()
                }
            };
            data.push(RowOfTwoStrings {
                key: "reason",
                value: reason.as_str(),
            });

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
        HealthCheckFailureDetails::NoActivePortListener(ref details) => details.reason.clone(),
        HealthCheckFailureDetails::NoActiveProtocolListener(ref details) => details.reason.clone(),
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
        HealthCheckFailureDetails::AlarmCheck(ClusterAlarmCheckDetails { reason: _, alarms }) => {
            for alarm in alarms {
                let key = format!("alarm in effect on node {}", alarm.node);
                let value = alarm.resource;
                tb.push_record([key.as_str(), value.as_str()]);
            }
        }
        HealthCheckFailureDetails::NodeIsQuorumCritical(QuorumCriticalityCheckDetails {
            reason: _,
            queues,
        }) => {
            for q in queues {
                let key = "affected queue, stream or internal component";
                let value = q.readable_name;
                tb.push_record([key, value.as_str()]);
            }
        }
        HealthCheckFailureDetails::NoActivePortListener(details) => {
            tb.push_record(["inactive port", details.inactive_port.to_string().as_str()]);
        }
        HealthCheckFailureDetails::NoActiveProtocolListener(details) => {
            tb.push_record([
                "inactive protocol",
                details.inactive_protocol.to_string().as_str(),
            ]);
        }
    };

    tb.build()
}

pub(crate) fn memory_breakdown(breakdown: NodeMemoryBreakdown) -> Table {
    // There is no easy way to transpose an existing table in Tabled, so…
    let atom_table_val = breakdown.atom_table;
    let allocated_but_unused_val = breakdown.allocated_but_unused;
    let binary_heap_val = breakdown.binary_heap;
    let classic_queue_procs_val = breakdown.classic_queue_procs;
    let code_val = breakdown.code;
    let connection_channels_val = breakdown.connection_channels;
    let connection_readers_val = breakdown.connection_readers;
    let connection_writers_val = breakdown.connection_writers;
    let connection_other_val = breakdown.connection_other;
    let management_db_val = breakdown.management_db;
    let message_indices_val = breakdown.message_indices;
    let metadata_store_val = breakdown.metadata_store;
    let metadata_store_ets_tables_val = breakdown.metadata_store_ets_tables;
    let metrics_val = breakdown.metrics;
    let mnesia_val = breakdown.mnesia;
    let other_ets_tables_val = breakdown.other_ets_tables;
    let other_system_val = breakdown.other_system;
    let other_procs_val = breakdown.other_procs;
    let quorum_queue_procs_val = breakdown.quorum_queue_procs;
    let quorum_queue_ets_tables_val = breakdown.quorum_queue_ets_tables;
    let plugins_val = breakdown.plugins;
    let reserved_but_unallocated_val = breakdown.reserved_but_unallocated;
    let stream_queue_procs_val = breakdown.stream_queue_procs;
    let stream_queue_replica_reader_procs_val = breakdown.stream_queue_replica_reader_procs;
    let stream_queue_coordinator_procs_val = breakdown.stream_queue_coordinator_procs;

    let total_per_rss_val = breakdown.total.rss;
    let total_allocated_val = breakdown.total.allocated;

    let mut data: Vec<RowOfTwoStrings<u64>> = vec![
        RowOfTwoStrings {
            key: "Total (RSS)",
            value: &total_per_rss_val,
        },
        RowOfTwoStrings {
            key: "Total (allocated by the runtime)",
            value: &total_allocated_val,
        },
        RowOfTwoStrings {
            key: "Atom table",
            value: &atom_table_val,
        },
        RowOfTwoStrings {
            key: "Allocated but unused",
            value: &allocated_but_unused_val,
        },
        RowOfTwoStrings {
            key: "Binary heap",
            value: &binary_heap_val,
        },
        RowOfTwoStrings {
            key: "Classic queue processes",
            value: &classic_queue_procs_val,
        },
        RowOfTwoStrings {
            key: "Code ",
            value: &code_val,
        },
        RowOfTwoStrings {
            key: "AMQP 0-9-1 channels",
            value: &connection_channels_val,
        },
        RowOfTwoStrings {
            key: "Client connections: reader processes",
            value: &connection_readers_val,
        },
        RowOfTwoStrings {
            key: "Client connections: writer processes",
            value: &connection_writers_val,
        },
        RowOfTwoStrings {
            key: "Client connections: others processes",
            value: &connection_other_val,
        },
        RowOfTwoStrings {
            key: "Management stats database",
            value: &management_db_val,
        },
        RowOfTwoStrings {
            key: "Message store indices",
            value: &message_indices_val,
        },
        RowOfTwoStrings {
            key: "Metadata store",
            value: &metadata_store_val,
        },
        RowOfTwoStrings {
            key: "Metadata store ETS tables",
            value: &metadata_store_ets_tables_val,
        },
        RowOfTwoStrings {
            key: "Metrics data",
            value: &metrics_val,
        },
        RowOfTwoStrings {
            key: "Mnesia",
            value: &mnesia_val,
        },
        RowOfTwoStrings {
            key: "Other (ETS tables)",
            value: &other_ets_tables_val,
        },
        RowOfTwoStrings {
            key: "Other (used by the runtime)",
            value: &other_system_val,
        },
        RowOfTwoStrings {
            key: "Other processes",
            value: &other_procs_val,
        },
        RowOfTwoStrings {
            key: "Quorum queue replica processes",
            value: &quorum_queue_procs_val,
        },
        RowOfTwoStrings {
            key: "Quorum queue ETS tables",
            value: &quorum_queue_ets_tables_val,
        },
        RowOfTwoStrings {
            key: "Plugins and their data",
            value: &plugins_val,
        },
        RowOfTwoStrings {
            key: "Reserved by the kernel but unallocated",
            value: &reserved_but_unallocated_val,
        },
        RowOfTwoStrings {
            key: "Stream replica processes",
            value: &stream_queue_procs_val,
        },
        RowOfTwoStrings {
            key: "Stream replica reader processes",
            value: &stream_queue_replica_reader_procs_val,
        },
        RowOfTwoStrings {
            key: "Stream coordinator processes",
            value: &stream_queue_coordinator_procs_val,
        },
    ];
    // Note: this is descending ordering
    data.sort_by(|a, b| b.value.cmp(a.value));
    let tb = Table::builder(data);
    tb.build()
}
