use serde::{Deserialize};
use reqwest;

pub type Error = reqwest::Error;

pub type Result<T> = std::result::Result<T, reqwest::Error>;

#[derive(Debug, Deserialize)]
pub struct ClusterNode {
    name: String,
    uptime: u32,
}