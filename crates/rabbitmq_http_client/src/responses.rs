use reqwest;
use serde::Deserialize;
use serde_aux::prelude::*;

pub type Error = reqwest::Error;
pub type Result<T> = std::result::Result<T, reqwest::Error>;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct VirtualHostMetadata {
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub default_queue_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct VirtualHost {
    pub name: String,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub default_queue_type: Option<String>,
    pub metadata: VirtualHostMetadata,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct User {
    pub name: String,
    pub tags: Vec<String>,
    pub password_hash: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Connection {
    pub name: String,
    pub node: String,
    pub state: String,
    pub protocol: String,
    #[serde(rename(deserialize = "user"))]
    pub username: String,
    pub connected_at: u64,
    #[serde(rename(deserialize = "host"))]
    pub server_hostname: String,
    #[serde(rename(deserialize = "port"))]
    pub server_port: u32,
    #[serde(rename(deserialize = "peer_host"))]
    pub client_hostname: String,
    #[serde(rename(deserialize = "peer_port"))]
    pub client_port: u32,
    pub channel_max: u16,
    #[serde(rename(deserialize = "channels"))]
    pub channel_count: u16,
    pub client_properties: ClientProperties,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ClientProperties {
    pub connection_name: String,
    pub platform: String,
    pub product: String,
    pub version: String,
    pub capabilities: ClientCapabilities,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ClientCapabilities {
    pub authentication_failure_close: bool,
    #[serde(rename(deserialize = "basic.nack"))]
    pub basic_nack: bool,
    #[serde(rename(deserialize = "connection.blocked"))]
    pub connection_blocked: bool,
    #[serde(rename(deserialize = "consumer_cancel_notify"))]
    pub consumer_cancel_notify: bool,
    #[serde(rename(deserialize = "exchange_exchange_bindings"))]
    pub exchange_to_exchange_bindings: bool,
    pub publisher_confirms: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Channel {
    #[serde(rename(deserialize = "number"))]
    pub id: u32,
    pub name: String,
    pub connection_details: ConnectionDetails,
    pub vhost: String,
    pub state: String,
    pub consumer_count: u32,
    #[serde(rename(deserialize = "confirm"))]
    pub has_publisher_confirms_enabled: bool,
    pub prefetch_count: u32,
    pub messages_unacknowledged: u32,
    pub messages_unconfirmed: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ConnectionDetails {
    pub name: String,
    #[serde(rename(deserialize = "peer_host"))]
    pub client_hostname: String,
    #[serde(rename(deserialize = "peer_port"))]
    pub client_port: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Consumer {
    pub consumer_tag: String,
    pub active: bool,
    pub exclusive: bool,
    #[serde(rename(deserialize = "ack_required"))]
    pub manual_ack: bool,
    pub queue: NameAndVirtualHost,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct NameAndVirtualHost {
    pub name: String,
    pub vhost: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ClusterNode {
    pub name: String,
    pub uptime: u32,
    pub run_queue: u32,
    pub processors: u32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub os_pid: u32,
    pub fd_total: u32,
    #[serde(rename(deserialize = "proc_total"))]
    pub total_erlang_processes: u32,
    pub sockets_total: u32,
    #[serde(rename(deserialize = "mem_limit"))]
    pub memory_high_watermark: u64,
    #[serde(rename(deserialize = "mem_alarm"))]
    pub has_memory_alarm_in_effect: bool,
    #[serde(rename(deserialize = "disk_free_limit"))]
    pub free_disk_space_low_watermark: u64,
    #[serde(rename(deserialize = "disk_free_alarm"))]
    pub has_free_disk_space_alarm_in_effect: bool,
    pub rates_mode: String,
}
