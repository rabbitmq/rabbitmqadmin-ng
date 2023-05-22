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
