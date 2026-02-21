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

use rabbitmqadmin::config::Scheme;
use std::str::FromStr;

#[test]
fn test_scheme_default_is_http() {
    assert_eq!(Scheme::default(), Scheme::Http);
}

#[test]
fn test_scheme_from_str_http() {
    assert_eq!(Scheme::from_str("http").unwrap(), Scheme::Http);
    assert_eq!(Scheme::from_str("HTTP").unwrap(), Scheme::Http);
    assert_eq!(Scheme::from_str("Http").unwrap(), Scheme::Http);
}

#[test]
fn test_scheme_from_str_https() {
    assert_eq!(Scheme::from_str("https").unwrap(), Scheme::Https);
    assert_eq!(Scheme::from_str("HTTPS").unwrap(), Scheme::Https);
    assert_eq!(Scheme::from_str("Https").unwrap(), Scheme::Https);
}

#[test]
fn test_scheme_from_str_invalid() {
    assert!(Scheme::from_str("ftp").is_err());
    assert!(Scheme::from_str("").is_err());
    assert!(Scheme::from_str("httpss").is_err());
}

#[test]
fn test_scheme_display() {
    assert_eq!(Scheme::Http.to_string(), "http");
    assert_eq!(Scheme::Https.to_string(), "https");
}

#[test]
fn test_scheme_is_https() {
    assert!(!Scheme::Http.is_https());
    assert!(Scheme::Https.is_https());
}

#[test]
fn test_scheme_as_str() {
    assert_eq!(Scheme::Http.as_str(), "http");
    assert_eq!(Scheme::Https.as_str(), "https");
}

#[test]
fn test_scheme_from_str_ref() {
    assert_eq!(Scheme::from("http"), Scheme::Http);
    assert_eq!(Scheme::from("https"), Scheme::Https);
    assert_eq!(Scheme::from("invalid"), Scheme::Http); // Falls back to default
}

#[test]
fn test_scheme_serde_roundtrip() {
    let http = Scheme::Http;
    let json = serde_json::to_string(&http).unwrap();
    assert_eq!(json, "\"http\"");
    let back: Scheme = serde_json::from_str(&json).unwrap();
    assert_eq!(back, Scheme::Http);

    let https = Scheme::Https;
    let json = serde_json::to_string(&https).unwrap();
    assert_eq!(json, "\"https\"");
    let back: Scheme = serde_json::from_str(&json).unwrap();
    assert_eq!(back, Scheme::Https);
}

#[test]
fn test_scheme_copy_clone() {
    let s1 = Scheme::Http;
    let s2 = s1; // Copy
    let s3 = s1.clone(); // Clone
    assert_eq!(s1, s2);
    assert_eq!(s1, s3);
}
