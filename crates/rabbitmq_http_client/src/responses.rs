use reqwest;
use serde::Deserialize;
use serde_aux::prelude::*;

pub type Error = reqwest::Error;
pub type Result<T> = std::result::Result<T, reqwest::Error>;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct VirtualHostMetadata {
    tags: Option<Vec<String>>,
    description: Option<String>,
    default_queue_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct VirtualHost {
    name: String,
    tags: Option<Vec<String>>,
    description: Option<String>,
    default_queue_type: Option<String>,
    metadata: VirtualHostMetadata,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct User {
    name: String,
    tags: Vec<String>,
    password_hash: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Connection {
    name: String,
    node: String,
    state: String,
    protocol: String,
    #[serde(rename(deserialize = "user"))]
    username: String,
    connected_at: u64,
    #[serde(rename(deserialize = "host"))]
    server_hostname: String,
    #[serde(rename(deserialize = "port"))]
    server_port: u32,
    #[serde(rename(deserialize = "peer_host"))]
    client_hostname: String,
    #[serde(rename(deserialize = "peer_port"))]
    client_port: u32,
    channel_max: u16,
    #[serde(rename(deserialize = "channels"))]
    channel_count: u16,
    client_properties: ClientProperties,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ClientProperties {
    connection_name: String,
    platform: String,
    product: String,
    version: String,
    capabilities: ClientCapabilities,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ClientCapabilities {
    authentication_failure_close: bool,
    #[serde(rename(deserialize = "basic.nack"))]
    basic_nack: bool,
    #[serde(rename(deserialize = "connection.blocked"))]
    connection_blocked: bool,
    #[serde(rename(deserialize = "consumer_cancel_notify"))]
    consumer_cancel_notify: bool,
    #[serde(rename(deserialize = "exchange_exchange_bindings"))]
    exchange_to_exchange_bindings: bool,
    publisher_confirms: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Channel {
    #[serde(rename(deserialize = "number"))]
    id: u32,
    name: String,
    connection_details: ConnectionDetails,
    vhost: String,
    state: String,
    consumer_count: u32,
    #[serde(rename(deserialize = "confirm"))]
    has_publisher_confirms_enabled: bool,
    prefetch_count: u32,
    messages_unacknowledged: u32,
    messages_unconfirmed: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ConnectionDetails {
    name: String,
    #[serde(rename(deserialize = "peer_host"))]
    client_hostname: String,
    #[serde(rename(deserialize = "peer_port"))]
    client_port: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Consumer {
    consumer_tag: String,
    active: bool,
    exclusive: bool,
    #[serde(rename(deserialize = "ack_required"))]
    manual_ack: bool,
    queue: NameAndVirtualHost
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct NameAndVirtualHost {
    name: String,
    vhost: String
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ClusterNode {
    name: String,
    uptime: u32,
    run_queue: u32,
    processors: u32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    os_pid: u32,
    fd_total: u32,
    #[serde(rename(deserialize = "proc_total"))]
    total_erlang_processes: u32,
    sockets_total: u32,
    #[serde(rename(deserialize = "mem_limit"))]
    memory_high_watermark: u64,
    #[serde(rename(deserialize = "mem_alarm"))]
    has_memory_alarm_in_effect: bool,
    #[serde(rename(deserialize = "disk_free_limit"))]
    free_disk_space_low_watermark: u64,
    #[serde(rename(deserialize = "disk_free_alarm"))]
    has_free_disk_space_alarm_in_effect: bool,
    rates_mode: String,
}
