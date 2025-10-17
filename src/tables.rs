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
use rabbitmq_http_client::error::ErrorDetails;
use rabbitmq_http_client::formatting::*;
use rabbitmq_http_client::password_hashing::HashingError;
use rabbitmq_http_client::responses::{
    ClusterAlarmCheckDetails, HealthCheckFailureDetails, NodeMemoryBreakdown, Overview,
    QuorumCriticalityCheckDetails, SchemaDefinitionSyncStatus,
};
use reqwest::StatusCode;
use std::{error::Error, fmt};
use tabled::settings::Panel;
use tabled::{Table, Tabled};
use url::Url;

fn build_table_with_header<T: Tabled>(data: Vec<T>, header: &str) -> Table {
    let mut table = Table::builder(data).build();
    table.with(Panel::header(header));
    table
}

fn build_simple_table<T: Tabled>(data: Vec<T>) -> Table {
    Table::builder(data).build()
}

fn build_request_failure_table(result: &str, reason: &str) -> Table {
    let data = vec![
        RowOfTwo {
            key: "result",
            value: result,
        },
        RowOfTwo {
            key: "reason",
            value: reason,
        },
    ];
    build_simple_table(data)
}

#[derive(Debug, Tabled)]
struct OverviewRow<'a> {
    key: &'a str,
    value: String,
}

#[derive(Debug, Tabled)]
struct RowOfTwo<'a, T>
where
    T: ?Sized + fmt::Display,
{
    key: &'a str,
    value: &'a T,
}

#[derive(Debug, Tabled)]
struct MemoryBreakdownRow<'a> {
    key: &'a str,
    #[tabled(skip)]
    comparable: f64,
    percentage: &'a str,
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
        OverviewRow {
            key: "Connections (total)",
            value: ov.object_totals.connections.to_string(),
        },
        OverviewRow {
            key: "AMQP 0-9-1 channels (total)",
            value: ov.object_totals.channels.to_string(),
        },
        OverviewRow {
            key: "Queues and streams (total)",
            value: ov.object_totals.queues.to_string(),
        },
        OverviewRow {
            key: "Consumers (total)",
            value: ov.object_totals.consumers.to_string(),
        },
        OverviewRow {
            key: "Messages (total)",
            value: ov.queue_totals.messages.to_string(),
        },
        OverviewRow {
            key: "Messages ready for delivery (total)",
            value: ov.queue_totals.messages_ready_for_delivery.to_string(),
        },
        OverviewRow {
            key: "Messages delivered but unacknowledged by consumers (total)",
            value: ov
                .queue_totals
                .messages_delivered_but_unacknowledged_by_consumers
                .to_string(),
        },
        OverviewRow {
            key: "Publishing (ingress) rate (global)",
            value: display_option_details_rate(&ov.message_stats.publishing_details),
        },
        OverviewRow {
            key: "Publishing confirm rate (global)",
            value: display_option_details_rate(&ov.message_stats.publisher_confirmation_details),
        },
        OverviewRow {
            key: "Consumer delivery (egress) rate (global)",
            value: display_option_details_rate(&ov.message_stats.delivery_details),
        },
        OverviewRow {
            key: "Consumer delivery in automatic acknowledgement mode rate (global)",
            value: display_option_details_rate(
                &ov.message_stats
                    .delivery_with_automatic_acknowledgement_details,
            ),
        },
        OverviewRow {
            key: "Consumer acknowledgement rate (global)",
            value: display_option_details_rate(&ov.message_stats.consumer_acknowledgement_details),
        },
        OverviewRow {
            key: "Unroutable messages: returned-to-publisher rate (global)",
            value: display_option_details_rate(
                &ov.message_stats.unroutable_returned_message_details,
            ),
        },
        OverviewRow {
            key: "Unroutable messages: dropped rate (global)",
            value: display_option_details_rate(
                &ov.message_stats.unroutable_dropped_message_details,
            ),
        },
        OverviewRow {
            key: "Cluster tags",
            value: display_tag_map_option(&ov.cluster_tags),
        },
        OverviewRow {
            key: "Node tags",
            value: display_tag_map_option(&ov.node_tags),
        },
    ];
    build_table_with_header(data, "Overview")
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
    build_table_with_header(
        data,
        "Entity (connections, queues, etc) churn over the most recent sampling period",
    )
}

pub fn show_salted_and_hashed_value(value: String) -> Table {
    let data = vec![RowOfTwo {
        key: "password hash",
        value: value.as_str(),
    }];
    build_table_with_header(data, "Result")
}

pub fn schema_definition_sync_status(status: SchemaDefinitionSyncStatus) -> Table {
    let operating_mode_s = &status.operating_mode.into();
    let state_s = &status.state.into();
    let upstream_endpoints_s = &status.upstream_endpoints.into();
    let mut data = vec![
        RowOfTwo {
            key: "node",
            value: &status.node,
        },
        RowOfTwo {
            key: "operating mode",
            value: operating_mode_s,
        },
        RowOfTwo {
            key: "state",
            value: state_s,
        },
        RowOfTwo {
            key: "upstream endpoints",
            value: upstream_endpoints_s,
        },
        RowOfTwo {
            key: "upstream username",
            value: &status.upstream_username,
        },
    ];

    let last_connection_time_s: String;
    if let Some(stamp) = &status.last_connection_completion_timestamp {
        last_connection_time_s = stamp.to_string();
        data.push(RowOfTwo {
            key: "last connection time time",
            value: &last_connection_time_s,
        })
    }

    let last_sync_request_s: String;
    if let Some(stamp) = &status.last_sync_request_timestamp {
        last_sync_request_s = stamp.to_string();
        data.push(RowOfTwo {
            key: "last sync request time",
            value: &last_sync_request_s,
        })
    }

    let sync_duration_s: String;
    if let Some(stamp) = &status.last_sync_duration {
        sync_duration_s = stamp.to_string();
        data.push(RowOfTwo {
            key: "last sync duration (in ms)",
            value: &sync_duration_s,
        })
    }

    build_table_with_header(data, "Schema Definition Sync Status")
}

pub fn failure_details(error: &HttpClientError) -> Table {
    match error {
        HttpClientError::MissingProperty { argument } => {
            let message = format!("Missing value for property (field) {}", argument);
            let data = vec![
                RowOfTwo {
                    key: "result",
                    value: "request was not executed",
                },
                RowOfTwo {
                    key: "message",
                    value: &message,
                },
            ];

            let tb = Table::builder(data);
            tb.build()
        }
        HttpClientError::UnsupportedArgumentValue { property } => {
            let message = format!(
                "Unsupported argument value for property (field) {}",
                property
            );
            let data = vec![
                RowOfTwo {
                    key: "result",
                    value: "request was not executed",
                },
                RowOfTwo {
                    key: "message",
                    value: &message,
                },
            ];

            let tb = Table::builder(data);
            tb.build()
        }
        HttpClientError::ClientErrorResponse {
            status_code,
            url,
            body,
            error_details,
            ..
        } => generic_failed_request_details(status_code, url, body, error_details),
        HttpClientError::ServerErrorResponse {
            status_code,
            url,
            body,
            error_details,
            ..
        } => generic_failed_request_details(status_code, url, body, error_details),
        HttpClientError::HealthCheckFailed {
            status_code,
            path,
            details,
        } => {
            let path_s = path.clone();
            let status_code_s = format!("{}", status_code);
            let mut data = vec![
                RowOfTwo {
                    key: "result",
                    value: "request failed",
                },
                RowOfTwo {
                    key: "status_code",
                    value: status_code_s.as_str(),
                },
                RowOfTwo {
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
                HealthCheckFailureDetails::NoActiveProtocolListeners(details) => {
                    details.reason.clone()
                }
            };
            data.push(RowOfTwo {
                key: "reason",
                value: reason.as_str(),
            });

            let tb = Table::builder(data);
            tb.build()
        }
        HttpClientError::NotFound => {
            let status_code_s = format!("{}", StatusCode::NOT_FOUND);
            let data = vec![
                RowOfTwo {
                    key: "result",
                    value: "request failed",
                },
                RowOfTwo {
                    key: "status_code",
                    value: status_code_s.as_str(),
                },
            ];
            build_simple_table(data)
        }
        HttpClientError::MultipleMatchingBindings => {
            let data = vec![
                RowOfTwo {
                    key: "result",
                    value: "request failed",
                },
                RowOfTwo {
                    key: "status_code",
                    value: StatusCode::CONFLICT.as_str(),
                },
                RowOfTwo {
                    key: "reason",
                    value: "multiple bindings found between the source and destination, please specify a --routing-key of the target binding",
                },
            ];
            build_simple_table(data)
        }
        HttpClientError::InvalidHeaderValue { .. } => {
            build_request_failure_table("request failed", "invalid HTTP request header value")
        }
        HttpClientError::IncompatibleBody { error, .. } => {
            let reason = format!(
                "response body is not compatible with the requested data type: {}",
                error
            );
            build_request_failure_table("request failed", &reason)
        }
        HttpClientError::ParsingError { message } => {
            let data = vec![
                RowOfTwo {
                    key: "result",
                    value: "request failed",
                },
                RowOfTwo {
                    key: "reason",
                    value: message.as_str(),
                },
            ];
            build_simple_table(data)
        }
        HttpClientError::RequestError {
            error,
            backtrace: _,
        } => {
            let reason = format!("HTTP API request failed: {}", error);
            let mut data = vec![
                RowOfTwo {
                    key: "result",
                    value: "request failed",
                },
                RowOfTwo {
                    key: "reason",
                    value: reason.as_str(),
                },
            ];

            let underlying_error1 = match error.source() {
                Some(source) => source.to_string(),
                None => "(none)".to_string(),
            };
            let underlying_error2 = match error.source() {
                Some(err) => match err.source() {
                    None => "(none)".to_string(),
                    Some(err2) => {
                        format!("{}", err2)
                    }
                },
                None => "(none)".to_string(),
            };
            let underlying_error3 = match error.source() {
                Some(err) => match err.source() {
                    None => "(none)".to_string(),
                    Some(err2) => match err2.source() {
                        None => "(none)".to_string(),
                        Some(err3) => {
                            format!("{}", err3)
                        }
                    },
                },
                None => "(none)".to_string(),
            };

            data.push(RowOfTwo {
                key: "underlying error",
                value: &underlying_error1,
            });
            data.push(RowOfTwo {
                key: "underlying error",
                value: &underlying_error2,
            });
            data.push(RowOfTwo {
                key: "underlying error",
                value: &underlying_error3,
            });

            let tb = Table::builder(data);
            tb.build()
        }
        HttpClientError::Other => {
            let data = vec![
                RowOfTwo {
                    key: "result",
                    value: "request failed",
                },
                RowOfTwo {
                    key: "reason",
                    value: "(not available)",
                },
            ];

            build_simple_table(data)
        }
    }
}

fn generic_failed_request_details(
    status_code: &StatusCode,
    url: &Option<Url>,
    body: &Option<String>,
    error_details: &Option<ErrorDetails>,
) -> Table {
    let status_code_s = status_code.to_string();
    let url_s = url.clone().unwrap().to_string();
    let body_s = body.clone().unwrap_or("N/A".to_string());

    let mut data = vec![
        RowOfTwo {
            key: "result",
            value: "request failed",
        },
        RowOfTwo {
            key: "status_code",
            value: status_code_s.as_str(),
        },
        RowOfTwo {
            key: "url",
            value: url_s.as_str(),
        },
        RowOfTwo {
            key: "body",
            value: body_s.as_str(),
        },
    ];

    if let Some(details) = error_details
        && let Some(reason) = details.reason()
    {
        data.push(RowOfTwo {
            key: "error",
            value: reason,
        });
    }

    build_simple_table(data)
}

pub fn hashing_error_details(error: &HashingError) -> Table {
    build_simple_table(vec![
        RowOfTwo {
            key: "result",
            value: "hashing failed",
        },
        RowOfTwo {
            key: "details",
            value: &error.to_string(),
        },
    ])
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
        HealthCheckFailureDetails::NoActiveProtocolListeners(ref details) => details.reason.clone(),
    };
    let code_str = format!("{}", status_code);

    let vec = vec![
        RowOfTwo {
            key: "result",
            value: "health check failed",
        },
        RowOfTwo {
            key: "path",
            value: path,
        },
        RowOfTwo {
            key: "status_code",
            value: code_str.as_str(),
        },
        RowOfTwo {
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
        HealthCheckFailureDetails::NoActiveProtocolListeners(details) => tb.push_record([
            "inactive protocols",
            details.inactive_protocols.join(", ").as_str(),
        ]),
    };

    tb.build()
}

pub(crate) fn memory_breakdown_in_bytes(breakdown: NodeMemoryBreakdown) -> Table {
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

    let mut data: Vec<RowOfTwo<u64>> = vec![
        RowOfTwo {
            key: "Total (RSS)",
            value: &total_per_rss_val,
        },
        RowOfTwo {
            key: "Total (allocated by the runtime)",
            value: &total_allocated_val,
        },
        RowOfTwo {
            key: "Atom table",
            value: &atom_table_val,
        },
        RowOfTwo {
            key: "Allocated but unused",
            value: &allocated_but_unused_val,
        },
        RowOfTwo {
            key: "Binary heap",
            value: &binary_heap_val,
        },
        RowOfTwo {
            key: "Classic queue processes",
            value: &classic_queue_procs_val,
        },
        RowOfTwo {
            key: "Code ",
            value: &code_val,
        },
        RowOfTwo {
            key: "AMQP 0-9-1 channels",
            value: &connection_channels_val,
        },
        RowOfTwo {
            key: "Client connections: reader processes",
            value: &connection_readers_val,
        },
        RowOfTwo {
            key: "Client connections: writer processes",
            value: &connection_writers_val,
        },
        RowOfTwo {
            key: "Client connections: others processes",
            value: &connection_other_val,
        },
        RowOfTwo {
            key: "Management stats database",
            value: &management_db_val,
        },
        RowOfTwo {
            key: "Message store indices",
            value: &message_indices_val,
        },
        RowOfTwo {
            key: "Metadata store",
            value: &metadata_store_val,
        },
        RowOfTwo {
            key: "Metadata store ETS tables",
            value: &metadata_store_ets_tables_val,
        },
        RowOfTwo {
            key: "Metrics data",
            value: &metrics_val,
        },
        RowOfTwo {
            key: "Mnesia",
            value: &mnesia_val,
        },
        RowOfTwo {
            key: "Other (ETS tables)",
            value: &other_ets_tables_val,
        },
        RowOfTwo {
            key: "Other (used by the runtime)",
            value: &other_system_val,
        },
        RowOfTwo {
            key: "Other processes",
            value: &other_procs_val,
        },
        RowOfTwo {
            key: "Quorum queue replica processes",
            value: &quorum_queue_procs_val,
        },
        RowOfTwo {
            key: "Quorum queue ETS tables",
            value: &quorum_queue_ets_tables_val,
        },
        RowOfTwo {
            key: "Plugins and their data",
            value: &plugins_val,
        },
        RowOfTwo {
            key: "Reserved by the kernel but unallocated",
            value: &reserved_but_unallocated_val,
        },
        RowOfTwo {
            key: "Stream replica processes",
            value: &stream_queue_procs_val,
        },
        RowOfTwo {
            key: "Stream replica reader processes",
            value: &stream_queue_replica_reader_procs_val,
        },
        RowOfTwo {
            key: "Stream coordinator processes",
            value: &stream_queue_coordinator_procs_val,
        },
    ];
    // Note: this is descending ordering
    data.sort_by(|a, b| b.value.cmp(a.value));
    build_simple_table(data)
}

pub(crate) fn memory_breakdown_in_percent(mut breakdown: NodeMemoryBreakdown) -> Table {
    // There is no easy way to transpose an existing table in Tabled, so…
    let atom_table_val = breakdown.atom_table_percentage();
    let atom_table_val_s = breakdown.atom_table_percentage_as_text();
    let allocated_but_unused_val = breakdown.allocated_but_unused_percentage();
    let allocated_but_unused_val_s = breakdown.allocated_but_unused_percentage_as_text();
    let binary_heap_val = breakdown.binary_heap_percentage();
    let binary_heap_val_s = breakdown.binary_heap_percentage_as_text();
    let classic_queue_procs_val = breakdown.classic_queue_procs_percentage();
    let classic_queue_procs_val_s = breakdown.classic_queue_procs_percentage_as_text();
    let code_val = breakdown.code_percentage();
    let code_val_s = breakdown.code_percentage_as_text();
    let connection_channels_val = breakdown.connection_channels_percentage();
    let connection_channels_val_s = breakdown.connection_channels_percentage_as_text();
    let connection_readers_val = breakdown.connection_readers_percentage();
    let connection_readers_val_s = breakdown.connection_readers_percentage_as_text();
    let connection_writers_val = breakdown.connection_writers_percentage();
    let connection_writers_val_s = breakdown.connection_writers_percentage_as_text();
    let connection_other_val = breakdown.connection_other_percentage();
    let connection_other_val_s = breakdown.connection_other_percentage_as_text();
    let management_db_val = breakdown.management_db_percentage();
    let management_db_val_s = breakdown.management_db_percentage_as_text();
    let message_indices_val = breakdown.message_indices_percentage();
    let message_indices_val_s = breakdown.message_indices_percentage_as_text();
    let metadata_store_val = breakdown.metadata_store_percentage();
    let metadata_store_val_s = breakdown.metadata_store_percentage_as_text();
    let metadata_store_ets_tables_val = breakdown.metadata_store_ets_tables_percentage();
    let metadata_store_ets_tables_val_s = breakdown.metadata_store_ets_tables_percentage_as_text();
    let metrics_val = breakdown.metrics_percentage();
    let metrics_val_s = breakdown.metrics_percentage_as_text();
    let mnesia_val = breakdown.mnesia_percentage();
    let mnesia_val_s = breakdown.mnesia_percentage_as_text();
    let other_ets_tables_val = breakdown.other_ets_tables_percentage();
    let other_ets_tables_val_s = breakdown.other_ets_tables_percentage_as_text();
    let other_system_val = breakdown.other_system_percentage();
    let other_system_val_s = breakdown.other_system_percentage_as_text();
    let other_procs_val = breakdown.other_procs_percentage();
    let other_procs_val_s = breakdown.other_procs_percentage_as_text();
    let quorum_queue_procs_val = breakdown.quorum_queue_procs_percentage();
    let quorum_queue_procs_val_s = breakdown.quorum_queue_procs_percentage_as_text();
    let quorum_queue_ets_tables_val = breakdown.quorum_queue_ets_tables_percentage();
    let quorum_queue_ets_tables_val_s = breakdown.quorum_queue_ets_tables_percentage_as_text();
    let plugins_val = breakdown.plugins_percentage();
    let plugins_val_s = breakdown.plugins_percentage_as_text();
    let reserved_but_unallocated_val = breakdown.reserved_but_unallocated_percentage();
    let reserved_but_unallocated_val_s = breakdown.reserved_but_unallocated_percentage_as_text();
    let stream_queue_procs_val = breakdown.stream_queue_procs_percentage();
    let stream_queue_procs_val_s = breakdown.stream_queue_procs_percentage_as_text();
    let stream_queue_replica_reader_procs_val =
        breakdown.stream_queue_replica_reader_procs_percentage();
    let stream_queue_replica_reader_procs_val_s =
        breakdown.stream_queue_replica_reader_procs_percentage_as_text();
    let stream_queue_coordinator_procs_val = breakdown.stream_queue_coordinator_procs_percentage();
    let stream_queue_coordinator_procs_val_s =
        breakdown.stream_queue_coordinator_procs_percentage_as_text();

    let mut data: Vec<MemoryBreakdownRow> = vec![
        MemoryBreakdownRow {
            key: "total",
            comparable: 100.0,
            percentage: "100%",
        },
        MemoryBreakdownRow {
            key: "Atom table",
            comparable: atom_table_val,
            percentage: &atom_table_val_s,
        },
        MemoryBreakdownRow {
            key: "Allocated but unused",
            comparable: allocated_but_unused_val,
            percentage: &allocated_but_unused_val_s,
        },
        MemoryBreakdownRow {
            key: "Binary heap",
            comparable: binary_heap_val,
            percentage: &binary_heap_val_s,
        },
        MemoryBreakdownRow {
            key: "Classic queue processes",
            comparable: classic_queue_procs_val,
            percentage: &classic_queue_procs_val_s,
        },
        MemoryBreakdownRow {
            key: "Code ",
            comparable: code_val,
            percentage: &code_val_s,
        },
        MemoryBreakdownRow {
            key: "AMQP 0-9-1 channels",
            comparable: connection_channels_val,
            percentage: &connection_channels_val_s,
        },
        MemoryBreakdownRow {
            key: "Client connections: reader processes",
            comparable: connection_readers_val,
            percentage: &connection_readers_val_s,
        },
        MemoryBreakdownRow {
            key: "Client connections: writer processes",
            comparable: connection_writers_val,
            percentage: &connection_writers_val_s,
        },
        MemoryBreakdownRow {
            key: "Client connections: others processes",
            comparable: connection_other_val,
            percentage: &connection_other_val_s,
        },
        MemoryBreakdownRow {
            key: "Management stats database",
            comparable: management_db_val,
            percentage: &management_db_val_s,
        },
        MemoryBreakdownRow {
            key: "Message store indices",
            comparable: message_indices_val,
            percentage: &message_indices_val_s,
        },
        MemoryBreakdownRow {
            key: "Metadata store",
            comparable: metadata_store_val,
            percentage: &metadata_store_val_s,
        },
        MemoryBreakdownRow {
            key: "Metadata store ETS tables",
            comparable: metadata_store_ets_tables_val,
            percentage: &metadata_store_ets_tables_val_s,
        },
        MemoryBreakdownRow {
            key: "Metrics data",
            comparable: metrics_val,
            percentage: &metrics_val_s,
        },
        MemoryBreakdownRow {
            key: "Mnesia",
            comparable: mnesia_val,
            percentage: &mnesia_val_s,
        },
        MemoryBreakdownRow {
            key: "Other (ETS tables)",
            comparable: other_ets_tables_val,
            percentage: &other_ets_tables_val_s,
        },
        MemoryBreakdownRow {
            key: "Other (used by the runtime)",
            comparable: other_system_val,
            percentage: &other_system_val_s,
        },
        MemoryBreakdownRow {
            key: "Other processes",
            comparable: other_procs_val,
            percentage: &other_procs_val_s,
        },
        MemoryBreakdownRow {
            key: "Quorum queue replica processes",
            comparable: quorum_queue_procs_val,
            percentage: &quorum_queue_procs_val_s,
        },
        MemoryBreakdownRow {
            key: "Quorum queue ETS tables",
            comparable: quorum_queue_ets_tables_val,
            percentage: &quorum_queue_ets_tables_val_s,
        },
        MemoryBreakdownRow {
            key: "Plugins and their data",
            comparable: plugins_val,
            percentage: &plugins_val_s,
        },
        MemoryBreakdownRow {
            key: "Reserved by the kernel but unallocated",
            comparable: reserved_but_unallocated_val,
            percentage: &reserved_but_unallocated_val_s,
        },
        MemoryBreakdownRow {
            key: "Stream replica processes",
            comparable: stream_queue_procs_val,
            percentage: &stream_queue_procs_val_s,
        },
        MemoryBreakdownRow {
            key: "Stream replica reader processes",
            comparable: stream_queue_replica_reader_procs_val,
            percentage: &stream_queue_replica_reader_procs_val_s,
        },
        MemoryBreakdownRow {
            key: "Stream coordinator processes",
            comparable: stream_queue_coordinator_procs_val,
            percentage: &stream_queue_coordinator_procs_val_s,
        },
    ];
    // Note: this is descending ordering
    data.sort_by(|a, b| b.comparable.total_cmp(&a.comparable));
    build_simple_table(data)
}

pub(crate) fn memory_breakdown_not_available() -> Table {
    let data = vec![
        RowOfTwo {
            key: "result",
            value: "not available",
        },
        RowOfTwo {
            key: "reason",
            value: "memory breakdown is not available (yet) on target node",
        },
    ];
    build_table_with_header(data, "Memory Breakdown")
}
