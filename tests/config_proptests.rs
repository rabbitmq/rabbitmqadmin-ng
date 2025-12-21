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

use proptest::prelude::*;
use rabbitmqadmin::config::{Scheme, SharedSettings};
use std::path::PathBuf;
use url::Url;

/// Normalizes a path prefix by ensuring it starts with a forward slash
fn normalize_path_prefix(prefix: &str) -> String {
    if prefix.starts_with('/') {
        prefix.to_string()
    } else {
        format!("/{}", prefix)
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Path prefix normalization always produces a string that starts with '/'
    #[test]
    fn path_prefix_always_starts_with_slash(prefix in "[a-zA-Z0-9/_-]{0,50}") {
        let normalized = normalize_path_prefix(&prefix);
        prop_assert!(normalized.starts_with('/'),
            "Normalized prefix '{}' doesn't start with '/'", normalized);
    }

    /// Normalizing a prefix that already starts with '/' is idempotent
    #[test]
    fn path_prefix_normalization_idempotent(prefix in "/[a-zA-Z0-9/_-]{0,50}") {
        let first = normalize_path_prefix(&prefix);
        let second = normalize_path_prefix(&first);
        prop_assert_eq!(&first, &second,
            "Normalization is not idempotent: '{}' != '{}'", first, second);
    }

    /// Prefixes without a leading slash get exactly one slash prepended
    #[test]
    fn path_prefix_adds_single_slash(prefix in "[a-zA-Z0-9_-]{1,50}") {
        prop_assume!(!prefix.starts_with('/'));
        let normalized = normalize_path_prefix(&prefix);
        let expected = format!("/{}", prefix);
        prop_assert_eq!(&normalized, &expected,
            "Expected '{}' but got '{}'", expected, normalized);
    }

    /// Empty strings get normalized to '/'
    #[test]
    fn path_prefix_empty_becomes_slash(_unit in 0u8..1) {
        let normalized = normalize_path_prefix("");
        prop_assert_eq!(normalized, "/", "Empty prefix should become '/'");
    }
}

fn scheme_strategy() -> impl Strategy<Value = Scheme> {
    prop_oneof![Just(Scheme::Http), Just(Scheme::Https),]
}

fn hostname_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        3 => Just("localhost".to_string()),
        3 => Just("127.0.0.1".to_string()),
        3 => Just("rabbitmq.example.com".to_string()),
        3 => Just("rabbit.local".to_string()),
        3 => Just("rmq.test".to_string()),
        2 => "[a-z]{3,10}\\.[a-z]{3,10}\\.[a-z]{2,3}",
    ]
}

fn port_strategy() -> impl Strategy<Value = u16> {
    prop_oneof![
        Just(15672u16),
        Just(15671u16),
        Just(80u16),
        Just(443u16),
        Just(8080u16),
        1024u16..65535u16,
    ]
}

fn path_prefix_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("/api".to_string()),
        Just("api".to_string()),
        Just("/".to_string()),
        Just("".to_string()),
        "/[a-z]{2,10}",
        "[a-z]{2,10}",
        "/[a-z]{2,10}/[a-z]{2,10}",
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property: Generated endpoint URL is always parseable
    #[test]
    fn endpoint_is_always_valid_url(
        scheme in scheme_strategy(),
        hostname in hostname_strategy(),
        port in port_strategy(),
        path_prefix in path_prefix_strategy(),
    ) {
        let settings = SharedSettings {
            scheme,
            hostname: Some(hostname.clone()),
            port: Some(port),
            path_prefix: path_prefix.clone(),
            base_uri: None,
            tls: scheme.is_https(),
            non_interactive: false,
            quiet: false,
            username: Some("guest".to_string()),
            password: Some("guest".to_string()),
            virtual_host: Some("/".to_string()),
            table_style: None,
            ca_certificate_bundle_path: None,
            client_certificate_file_path: None,
            client_private_key_file_path: None,
        };

        let endpoint = settings.endpoint();

        // The endpoint should be parseable as a URL
        let parsed = Url::parse(&endpoint);
        prop_assert!(parsed.is_ok(),
            "Failed to parse endpoint '{}' as URL: {:?}", endpoint, parsed.err());
    }

    /// Property: Endpoint URL always has normalized path prefix (starts with a slash)
    #[test]
    fn endpoint_path_always_starts_with_slash(
        scheme in scheme_strategy(),
        hostname in hostname_strategy(),
        port in port_strategy(),
        path_prefix in path_prefix_strategy(),
    ) {
        let settings = SharedSettings {
            scheme,
            hostname: Some(hostname.clone()),
            port: Some(port),
            path_prefix: path_prefix.clone(),
            base_uri: None,
            tls: scheme.is_https(),
            non_interactive: false,
            quiet: false,
            username: Some("guest".to_string()),
            password: Some("guest".to_string()),
            virtual_host: Some("/".to_string()),
            table_style: None,
            ca_certificate_bundle_path: None,
            client_certificate_file_path: None,
            client_private_key_file_path: None,
        };

        let endpoint = settings.endpoint();
        let parsed = Url::parse(&endpoint).unwrap();
        let path = parsed.path();

        prop_assert!(path.starts_with('/'),
            "Endpoint path '{}' doesn't start with '/' for prefix '{}'",
            path, path_prefix);
    }

    /// Property: Endpoint preserves scheme, hostname, and port correctly
    #[test]
    fn endpoint_preserves_components(
        scheme in scheme_strategy(),
        hostname in hostname_strategy(),
        port in port_strategy(),
    ) {
        let settings = SharedSettings {
            scheme,
            hostname: Some(hostname.clone()),
            port: Some(port),
            path_prefix: "/api".to_string(),
            base_uri: None,
            tls: scheme.is_https(),
            non_interactive: false,
            quiet: false,
            username: Some("guest".to_string()),
            password: Some("guest".to_string()),
            virtual_host: Some("/".to_string()),
            table_style: None,
            ca_certificate_bundle_path: None,
            client_certificate_file_path: None,
            client_private_key_file_path: None,
        };

        let endpoint = settings.endpoint();
        let parsed = Url::parse(&endpoint).unwrap();

        prop_assert_eq!(parsed.scheme(), scheme.as_str(),
            "Scheme mismatch in endpoint '{}'", endpoint);
        prop_assert_eq!(parsed.host_str(), Some(hostname.as_str()),
            "Hostname mismatch in endpoint '{}'", endpoint);

        // URL parser returns None for default ports (80 for http, 443 for https)
        // but the endpoint string always includes a port
        let expected_port = if (scheme == Scheme::Http && port == 80) || (scheme == Scheme::Https && port == 443) {
            None
        } else {
            Some(port)
        };
        prop_assert_eq!(parsed.port(), expected_port,
            "Port mismatch in endpoint '{}' (scheme={}, expected_port={:?})", endpoint, scheme, expected_port);
    }

    /// Property: Endpoint has no leading or trailing whitespace
    #[test]
    fn endpoint_has_no_whitespace(
        scheme in scheme_strategy(),
        hostname in hostname_strategy(),
        port in port_strategy(),
        path_prefix in path_prefix_strategy(),
    ) {
        let settings = SharedSettings {
            scheme,
            hostname: Some(hostname),
            port: Some(port),
            path_prefix,
            base_uri: None,
            tls: false,
            non_interactive: false,
            quiet: false,
            username: Some("guest".to_string()),
            password: Some("guest".to_string()),
            virtual_host: Some("/".to_string()),
            table_style: None,
            ca_certificate_bundle_path: None,
            client_certificate_file_path: None,
            client_private_key_file_path: None,
        };

        let endpoint = settings.endpoint();

        let trimmed = endpoint.trim();
        prop_assert_eq!(trimmed, endpoint.as_str(),
            "Endpoint has whitespace: '{}'", endpoint);
    }

    /// Property: Endpoint contains the scheme, hostname, and port in correct format
    #[test]
    fn endpoint_format_is_correct(
        scheme in scheme_strategy(),
        hostname in hostname_strategy(),
        port in port_strategy(),
    ) {
        let settings = SharedSettings {
            scheme,
            hostname: Some(hostname.clone()),
            port: Some(port),
            path_prefix: "/api".to_string(),
            base_uri: None,
            tls: false,
            non_interactive: false,
            quiet: false,
            username: Some("guest".to_string()),
            password: Some("guest".to_string()),
            virtual_host: Some("/".to_string()),
            table_style: None,
            ca_certificate_bundle_path: None,
            client_certificate_file_path: None,
            client_private_key_file_path: None,
        };

        let endpoint = settings.endpoint();

        // Should contain scheme://
        prop_assert!(endpoint.starts_with(&format!("{}://", scheme)),
            "Endpoint '{}' doesn't start with '{}://'", endpoint, scheme);

        // Should contain hostname
        prop_assert!(endpoint.contains(&hostname),
            "Endpoint '{}' doesn't contain hostname '{}'", endpoint, hostname);

        // Should contain :port
        prop_assert!(endpoint.contains(&format!(":{}", port)),
            "Endpoint '{}' doesn't contain port ':{}'", endpoint, port);
    }
}

// ============================================================================
// Priority 4: Configuration Merging Logic
// ============================================================================

use clap::{Arg, ArgAction, Command};

/// Helper to create a mock CLI parser for testing
fn create_test_parser() -> Command {
    Command::new("test")
        .arg(Arg::new("host").long("host"))
        .arg(
            Arg::new("port")
                .long("port")
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(Arg::new("username").long("username"))
        .arg(Arg::new("password").long("password"))
        .arg(Arg::new("vhost").long("vhost"))
        .arg(Arg::new("tls").long("tls").action(ArgAction::SetTrue))
        .arg(
            Arg::new("non_interactive")
                .long("non-interactive")
                .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("quiet").long("quiet").action(ArgAction::SetTrue))
        .arg(Arg::new("path_prefix").long("path-prefix"))
        .arg(Arg::new("base_uri").long("base-uri"))
        .arg(Arg::new("table_style").long("table-style"))
        .arg(
            Arg::new("ca_certificate_bundle_path")
                .long("ca-cert")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("client_certificate_file_path")
                .long("client-cert")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("client_private_key_file_path")
                .long("client-key")
                .value_parser(clap::value_parser!(PathBuf)),
        )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: CLI arguments always override config file defaults
    #[test]
    fn cli_args_override_config_defaults(
        cli_hostname in "[a-z]{5,10}\\.[a-z]{3,5}",
        config_hostname in "[a-z]{5,10}\\.[a-z]{3,5}",
        cli_port in 1024u16..65535u16,
        config_port in 1024u16..65535u16,
    ) {
        prop_assume!(cli_hostname != config_hostname);
        prop_assume!(cli_port != config_port);

        let parser = create_test_parser();
        let matches = parser.try_get_matches_from(vec![
            "test",
            "--host", &cli_hostname,
            "--port", &cli_port.to_string(),
        ]).unwrap();

        let config_defaults = SharedSettings {
            hostname: Some(config_hostname.clone()),
            port: Some(config_port),
            scheme: Scheme::Http,
            path_prefix: "/api".to_string(),
            tls: false,
            non_interactive: false,
            quiet: false,
            base_uri: None,
            username: Some("guest".to_string()),
            password: Some("guest".to_string()),
            virtual_host: Some("/".to_string()),
            table_style: None,
            ca_certificate_bundle_path: None,
            client_certificate_file_path: None,
            client_private_key_file_path: None,
        };

        let merged = SharedSettings::new_with_defaults(&matches, &config_defaults);

        prop_assert_eq!(merged.hostname, Some(cli_hostname.clone()),
            "CLI hostname should override config default");
        prop_assert_eq!(merged.port, Some(cli_port),
            "CLI port should override config default");
    }

    /// Property: When CLI doesn't provide values, config defaults are used
    #[test]
    fn config_defaults_used_when_no_cli_args(
        config_hostname in "[a-z]{5,10}\\.[a-z]{3,5}",
        config_port in 1024u16..65535u16,
        config_username in "[a-z]{5,10}",
    ) {
        let parser = create_test_parser();
        let matches = parser.try_get_matches_from(vec!["test"]).unwrap();

        let config_defaults = SharedSettings {
            hostname: Some(config_hostname.clone()),
            port: Some(config_port),
            username: Some(config_username.clone()),
            scheme: Scheme::Http,
            path_prefix: "/api".to_string(),
            tls: false,
            non_interactive: false,
            quiet: false,
            base_uri: None,
            password: Some("guest".to_string()),
            virtual_host: Some("/".to_string()),
            table_style: None,
            ca_certificate_bundle_path: None,
            client_certificate_file_path: None,
            client_private_key_file_path: None,
        };

        let merged = SharedSettings::new_with_defaults(&matches, &config_defaults);

        prop_assert_eq!(merged.hostname, Some(config_hostname),
            "Config hostname should be used");
        prop_assert_eq!(merged.port, Some(config_port),
            "Config port should be used");
        prop_assert_eq!(merged.username, Some(config_username),
            "Config username should be used");
    }

    /// Property: TLS flag properly sets HTTPS scheme
    #[test]
    fn tls_flag_sets_https_scheme(use_tls in proptest::bool::ANY) {
        let parser = create_test_parser();
        let args = if use_tls {
            vec!["test", "--tls"]
        } else {
            vec!["test"]
        };
        let matches = parser.try_get_matches_from(args).unwrap();

        let config_defaults = SharedSettings {
            scheme: Scheme::Http,
            hostname: Some("localhost".to_string()),
            port: Some(15672),
            path_prefix: "/api".to_string(),
            tls: false,
            non_interactive: false,
            quiet: false,
            base_uri: None,
            username: Some("guest".to_string()),
            password: Some("guest".to_string()),
            virtual_host: Some("/".to_string()),
            table_style: None,
            ca_certificate_bundle_path: None,
            client_certificate_file_path: None,
            client_private_key_file_path: None,
        };

        let merged = SharedSettings::new_with_defaults(&matches, &config_defaults);

        if use_tls {
            prop_assert_eq!(merged.scheme, Scheme::Https,
                "TLS flag should set scheme to https");
            prop_assert!(merged.tls, "TLS flag should set tls to true");
        }
    }

    /// Property: Port stays in valid u16 range after merging
    #[test]
    fn port_stays_in_valid_range(
        config_port in 1u16..65535u16,
        cli_port in 1u16..65535u16,
    ) {
        let parser = create_test_parser();
        let matches = parser.try_get_matches_from(vec![
            "test",
            "--port", &cli_port.to_string(),
        ]).unwrap();

        let config_defaults = SharedSettings {
            hostname: Some("localhost".to_string()),
            port: Some(config_port),
            scheme: Scheme::Http,
            path_prefix: "/api".to_string(),
            tls: false,
            non_interactive: false,
            quiet: false,
            base_uri: None,
            username: Some("guest".to_string()),
            password: Some("guest".to_string()),
            virtual_host: Some("/".to_string()),
            table_style: None,
            ca_certificate_bundle_path: None,
            client_certificate_file_path: None,
            client_private_key_file_path: None,
        };

        let merged = SharedSettings::new_with_defaults(&matches, &config_defaults);

        if let Some(port) = merged.port {
            prop_assert!(port > 0,
                "Port {} must be greater than 0", port);
        }
    }

    /// Property: Username and password are always set (use defaults if not provided)
    #[test]
    fn username_password_always_set(
        provide_username in proptest::bool::ANY,
        provide_password in proptest::bool::ANY,
        username in "[a-z]{5,10}",
        password in "[a-zA-Z0-9]{8,16}",
    ) {
        let parser = create_test_parser();
        let mut args = vec!["test"];
        if provide_username {
            args.push("--username");
            args.push(&username);
        }
        if provide_password {
            args.push("--password");
            args.push(&password);
        }
        let matches = parser.try_get_matches_from(args).unwrap();

        let config_defaults = SharedSettings {
            hostname: Some("localhost".to_string()),
            port: Some(15672),
            scheme: Scheme::Http,
            path_prefix: "/api".to_string(),
            tls: false,
            non_interactive: false,
            quiet: false,
            base_uri: None,
            username: Some("config_user".to_string()),
            password: Some("config_pass".to_string()),
            virtual_host: Some("/".to_string()),
            table_style: None,
            ca_certificate_bundle_path: None,
            client_certificate_file_path: None,
            client_private_key_file_path: None,
        };

        let merged = SharedSettings::new_with_defaults(&matches, &config_defaults);

        prop_assert!(merged.username.is_some(),
            "Username should always be set");
        prop_assert!(merged.password.is_some(),
            "Password should always be set");

        if provide_username {
            prop_assert_eq!(merged.username, Some(username),
                "CLI username should be used");
        } else {
            prop_assert_eq!(merged.username, Some("config_user".to_string()),
                "Config username should be used");
        }
    }

    /// Property: Virtual host defaults to "/" when not provided
    #[test]
    fn vhost_defaults_to_slash(provide_vhost in proptest::bool::ANY, vhost in "[a-z]{3,10}") {
        let parser = create_test_parser();
        let mut args = vec!["test"];
        if provide_vhost {
            args.push("--vhost");
            args.push(&vhost);
        }
        let matches = parser.try_get_matches_from(args).unwrap();

        let config_defaults = SharedSettings {
            hostname: Some("localhost".to_string()),
            port: Some(15672),
            scheme: Scheme::Http,
            path_prefix: "/api".to_string(),
            tls: false,
            non_interactive: false,
            quiet: false,
            base_uri: None,
            username: Some("guest".to_string()),
            password: Some("guest".to_string()),
            virtual_host: None,
            table_style: None,
            ca_certificate_bundle_path: None,
            client_certificate_file_path: None,
            client_private_key_file_path: None,
        };

        let merged = SharedSettings::new_with_defaults(&matches, &config_defaults);

        if provide_vhost {
            prop_assert_eq!(merged.virtual_host, Some(vhost),
                "CLI vhost should be used");
        } else {
            prop_assert_eq!(merged.virtual_host, Some("/".to_string()),
                "Vhost should default to /");
        }
    }

    /// Property: Boolean flags (non_interactive, quiet) are merged using OR
    #[test]
    fn boolean_flags_combine_with_or(
        cli_non_interactive in proptest::bool::ANY,
        config_non_interactive in proptest::bool::ANY,
        cli_quiet in proptest::bool::ANY,
        config_quiet in proptest::bool::ANY,
    ) {
        let parser = create_test_parser();
        let mut args = vec!["test"];
        if cli_non_interactive {
            args.push("--non-interactive");
        }
        if cli_quiet {
            args.push("--quiet");
        }
        let matches = parser.try_get_matches_from(args).unwrap();

        let config_defaults = SharedSettings {
            hostname: Some("localhost".to_string()),
            port: Some(15672),
            scheme: Scheme::Http,
            path_prefix: "/api".to_string(),
            tls: false,
            non_interactive: config_non_interactive,
            quiet: config_quiet,
            base_uri: None,
            username: Some("guest".to_string()),
            password: Some("guest".to_string()),
            virtual_host: Some("/".to_string()),
            table_style: None,
            ca_certificate_bundle_path: None,
            client_certificate_file_path: None,
            client_private_key_file_path: None,
        };

        let merged = SharedSettings::new_with_defaults(&matches, &config_defaults);

        // Boolean flags should use OR logic
        prop_assert_eq!(merged.non_interactive, cli_non_interactive || config_non_interactive,
            "non_interactive should be true if either CLI or config is true");
        prop_assert_eq!(merged.quiet, cli_quiet || config_quiet,
            "quiet should be true if either CLI or config is true");
    }
}
