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
use crate::constants::{
    DEFAULT_CONFIG_SECTION_NAME, DEFAULT_HOST, DEFAULT_HTTP_PORT, DEFAULT_HTTPS_PORT,
    DEFAULT_PASSWORD, DEFAULT_PATH_PREFIX, DEFAULT_SCHEME, DEFAULT_USERNAME, DEFAULT_VHOST,
    HTTPS_SCHEME,
};
use crate::output::TableStyle;
use clap::ArgMatches;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use url::Url;

/// A set of settings that must be set very early on.
/// More specifically, before the command line argument parser is
/// configured.
#[derive(Debug, Clone)]
pub struct PreFlightSettings {
    pub infer_subcommands: bool,
    pub infer_long_options: bool,
}

impl Default for PreFlightSettings {
    fn default() -> Self {
        Self {
            infer_long_options: true,
            infer_subcommands: false,
        }
    }
}

impl PreFlightSettings {
    /// Returns a set of [`PreFlightSettings`] that disable inference.
    /// Primarily meant to be used by/for the non-interactive mode.
    pub fn non_interactive() -> Self {
        Self {
            infer_long_options: false,
            infer_subcommands: false,
        }
    }
}

#[derive(Error, Debug)]
pub enum ConfigFileError {
    #[error("provided config file at '{0}' does not exist")]
    MissingFile(PathBuf),
    #[error(
        "provided configuration section (--node) '{0}' was not found in the configuration file"
    )]
    MissingConfigSection(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("failed to deserialize config file. Make sure it is valid TOML")]
    DeserializationError(#[from] toml::de::Error),
}

type ConfigurationMap<'a> = HashMap<String, SharedSettings>;

/// Represents a set of settings that can be set both via
/// the command line arguments and an optional configuration file.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SharedSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_uri: Option<String>,

    #[serde(default = "default_tls")]
    pub tls: bool,

    #[serde(default = "default_non_interactive")]
    pub non_interactive: bool,
    #[serde(default = "default_quiet")]
    pub quiet: bool,

    #[serde(default = "default_scheme")]
    pub scheme: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default = "default_path_prefix")]
    pub path_prefix: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_host: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_style: Option<TableStyle>,
}

impl SharedSettings {
    pub fn from_config_file(
        path: &Path,
        section_name: Option<String>,
    ) -> Result<Self, ConfigFileError> {
        let section = section_name.unwrap_or(DEFAULT_CONFIG_SECTION_NAME.to_string());

        from_local_path(path).and_then(|cm| {
            let err = Err(ConfigFileError::MissingConfigSection(section.clone()));
            match cm.get(section.as_str()) {
                None => err,
                Some(val) => Ok(val.clone()),
            }
        })
    }

    pub fn from_args_with_defaults(general_args: &ArgMatches, config_file_defaults: &Self) -> Self {
        let base_uri = general_args
            .get_one::<String>("base_uri")
            .cloned()
            .or(config_file_defaults.base_uri.clone());

        if let Some(s) = base_uri {
            let url = Url::parse(&s).unwrap();
            SharedSettings::new_from_uri_with_defaults(&url, general_args, config_file_defaults)
        } else {
            SharedSettings::new_with_defaults(general_args, config_file_defaults)
        }
    }

    pub fn from_args(general_args: &ArgMatches) -> Self {
        let base_uri = general_args.get_one::<String>("base_uri").cloned();

        if let Some(s) = base_uri {
            let url = Url::parse(&s).unwrap();
            SharedSettings::new_from_uri(&url, general_args)
        } else {
            SharedSettings::new(general_args)
        }
    }

    pub fn new_with_defaults(cli_args: &ArgMatches, config_file_defaults: &Self) -> Self {
        let default_hostname = DEFAULT_HOST.to_string();
        let should_use_tls = cli_args
            .get_one::<bool>("tls")
            .cloned()
            .unwrap_or(config_file_defaults.tls);
        let non_interactive = cli_args
            .get_one::<bool>("non_interactive")
            .cloned()
            .unwrap_or(false)
            || config_file_defaults.non_interactive;
        let quiet = cli_args.get_one::<bool>("quiet").cloned().unwrap_or(false)
            || config_file_defaults.quiet;
        let scheme = if should_use_tls { "https" } else { "http" };
        let hostname = cli_args
            .get_one::<String>("host")
            .cloned()
            .or(config_file_defaults.hostname.clone())
            .unwrap_or(default_hostname);
        let port: u16 = cli_args
            .get_one::<u16>("port")
            .cloned()
            .or(config_file_defaults.port)
            .or({
                if should_use_tls {
                    Some(DEFAULT_HTTPS_PORT)
                } else {
                    Some(DEFAULT_HTTP_PORT)
                }
            })
            .unwrap();
        let path_prefix = cli_args
            .get_one::<String>("path_prefix")
            .cloned()
            .or(Some(config_file_defaults.path_prefix.clone()))
            .unwrap_or(DEFAULT_PATH_PREFIX.to_owned());
        let username = cli_args
            .get_one::<String>("username")
            .cloned()
            .or(config_file_defaults.username.clone())
            .unwrap_or(DEFAULT_USERNAME.to_string());
        let password = cli_args
            .get_one::<String>("password")
            .cloned()
            .or(config_file_defaults.password.clone())
            .unwrap_or(DEFAULT_PASSWORD.to_string());
        let vhost = cli_args
            .get_one::<String>("vhost")
            .cloned()
            .or(config_file_defaults.virtual_host.clone())
            .unwrap_or(DEFAULT_VHOST.to_owned());
        let table_style = cli_args
            .get_one::<TableStyle>("table_style")
            .cloned()
            .or(Some(TableStyle::default()))
            .unwrap_or_default();

        Self {
            tls: should_use_tls,
            non_interactive,
            quiet,
            base_uri: None,
            scheme: scheme.to_string(),
            hostname: Some(hostname),
            port: Some(port),
            path_prefix: path_prefix.clone(),
            username: Some(username.to_string()),
            password: Some(password.to_string()),
            virtual_host: Some(vhost),
            table_style: Some(table_style),
        }
    }

    pub fn new(cli_args: &ArgMatches) -> Self {
        let default_hostname = DEFAULT_HOST.to_string();
        let should_use_tls = cli_args.get_one::<bool>("tls").cloned().unwrap_or(false);
        let non_interactive = cli_args
            .get_one::<bool>("non_interactive")
            .cloned()
            .unwrap_or(false)
            || default_non_interactive();
        let quiet = cli_args.get_one::<bool>("quiet").cloned().unwrap_or(false) || default_quiet();
        let scheme = if should_use_tls { "https" } else { "http" };
        let hostname = cli_args
            .get_one::<String>("host")
            .cloned()
            .unwrap_or(default_hostname);
        let port: u16 = cli_args
            .get_one::<u16>("port")
            .cloned()
            .or({
                if should_use_tls {
                    Some(DEFAULT_HTTPS_PORT)
                } else {
                    Some(DEFAULT_HTTP_PORT)
                }
            })
            .unwrap();
        let path_prefix = cli_args
            .get_one::<String>("path_prefix")
            .cloned()
            .unwrap_or(DEFAULT_PATH_PREFIX.to_owned());
        let username = cli_args
            .get_one::<String>("username")
            .cloned()
            .unwrap_or(DEFAULT_USERNAME.to_string());
        let password = cli_args
            .get_one::<String>("password")
            .cloned()
            .unwrap_or(DEFAULT_PASSWORD.to_string());
        let vhost = cli_args
            .get_one::<String>("vhost")
            .cloned()
            .unwrap_or(DEFAULT_VHOST.to_owned());
        let table_style = cli_args
            .get_one::<TableStyle>("table_style")
            .cloned()
            .or(Some(TableStyle::default()))
            .unwrap_or_default();

        Self {
            tls: should_use_tls,
            non_interactive,
            quiet,
            base_uri: None,
            scheme: scheme.to_string(),
            hostname: Some(hostname),
            port: Some(port),
            path_prefix: path_prefix.clone(),
            username: Some(username.to_string()),
            password: Some(password.to_string()),
            virtual_host: Some(vhost),
            table_style: Some(table_style),
        }
    }

    pub fn new_from_uri_with_defaults(
        url: &Url,
        cli_args: &ArgMatches,
        config_file_defaults: &Self,
    ) -> Self {
        let should_use_tls = cli_args.get_one::<bool>("tls").cloned().unwrap_or(false)
            || config_file_defaults.tls
            || url.scheme() == HTTPS_SCHEME;
        let non_interactive = cli_args
            .get_one::<bool>("non_interactive")
            .cloned()
            .unwrap_or(false)
            || config_file_defaults.non_interactive;
        let quiet = cli_args.get_one::<bool>("quiet").cloned().unwrap_or(false)
            || config_file_defaults.quiet;

        let scheme = url.scheme().to_string();
        let hostname = url.host_str().unwrap_or(DEFAULT_HOST).to_string();
        let port = url
            .port()
            .or({
                if should_use_tls {
                    Some(DEFAULT_HTTPS_PORT)
                } else {
                    Some(DEFAULT_HTTP_PORT)
                }
            })
            .unwrap_or(DEFAULT_HTTP_PORT);
        let path_prefix = cli_args
            .get_one::<String>("path_prefix")
            .cloned()
            .or(Some(config_file_defaults.path_prefix.clone()))
            .unwrap_or(DEFAULT_PATH_PREFIX.to_owned());
        let username = cli_args
            .get_one::<String>("username")
            .cloned()
            .or(config_file_defaults.username.clone())
            .unwrap_or(DEFAULT_USERNAME.to_string());
        let password = cli_args
            .get_one::<String>("password")
            .cloned()
            .or(config_file_defaults.password.clone())
            .unwrap_or(DEFAULT_PASSWORD.to_string());
        let vhost = cli_args
            .get_one::<String>("vhost")
            .cloned()
            .or(config_file_defaults.virtual_host.clone())
            .unwrap_or(DEFAULT_VHOST.to_owned());
        let table_style = cli_args
            .get_one::<TableStyle>("table_style")
            .cloned()
            .or(Some(TableStyle::default()))
            .unwrap_or_default();

        Self {
            tls: should_use_tls,
            non_interactive,
            quiet,
            base_uri: Some(url.to_string()),
            scheme: scheme.to_string(),
            hostname: Some(hostname),
            port: Some(port),
            path_prefix,
            username: Some(username),
            password: Some(password),
            virtual_host: Some(vhost),
            table_style: Some(table_style),
        }
    }

    pub fn new_from_uri(url: &Url, cli_args: &ArgMatches) -> Self {
        let should_use_tls =
            *cli_args.get_one::<bool>("tls").unwrap() || url.scheme() == HTTPS_SCHEME;
        let non_interactive = cli_args
            .get_one::<bool>("non_interactive")
            .cloned()
            .unwrap_or(default_non_interactive());
        let quiet = cli_args
            .get_one::<bool>("quiet")
            .cloned()
            .unwrap_or(default_quiet());

        let scheme = url.scheme().to_string();
        let hostname = url.host_str().unwrap_or(DEFAULT_HOST).to_string();
        let port = url
            .port()
            .or({
                if should_use_tls {
                    Some(DEFAULT_HTTPS_PORT)
                } else {
                    Some(DEFAULT_HTTP_PORT)
                }
            })
            .unwrap_or(DEFAULT_HTTP_PORT);
        let path_prefix = cli_args
            .get_one::<String>("path_prefix")
            .cloned()
            .unwrap_or(DEFAULT_PATH_PREFIX.to_owned());
        let username = cli_args
            .get_one::<String>("username")
            .cloned()
            .unwrap_or(DEFAULT_USERNAME.to_string());
        let password = cli_args
            .get_one::<String>("password")
            .cloned()
            .unwrap_or(DEFAULT_PASSWORD.to_string());
        let vhost = cli_args
            .get_one::<String>("vhost")
            .cloned()
            .unwrap_or(DEFAULT_VHOST.to_owned());
        let table_style = cli_args
            .get_one::<TableStyle>("table_style")
            .cloned()
            .or(Some(TableStyle::default()))
            .unwrap_or_default();

        Self {
            tls: should_use_tls,
            non_interactive,
            quiet,
            base_uri: Some(url.to_string()),
            scheme: scheme.to_string(),
            hostname: Some(hostname),
            port: Some(port),
            path_prefix,
            username: Some(username),
            password: Some(password),
            virtual_host: Some(vhost),
            table_style: Some(table_style),
        }
    }

    pub fn endpoint(&self) -> String {
        let prefix = if self.path_prefix.starts_with("/") {
            self.path_prefix.clone()
        } else {
            format!("/{}", self.path_prefix)
        };
        format!(
            "{}://{}:{}{}",
            self.scheme,
            self.hostname.as_ref().to_owned().unwrap(),
            self.port.unwrap(),
            prefix
        )
        .trim()
        .to_string()
    }
}

fn from_local_path(path: &Path) -> Result<ConfigurationMap, ConfigFileError> {
    let expanded_s = shellexpand::tilde(&path.to_string_lossy()).to_string();
    let expanded_path = PathBuf::from(&expanded_s);
    if expanded_path.exists() {
        read_from_local_path(&expanded_path)
    } else {
        Err(ConfigFileError::MissingFile((*expanded_path).to_path_buf()))
    }
}

fn read_from_local_path(path: &PathBuf) -> Result<ConfigurationMap, ConfigFileError> {
    let contents = std::fs::read_to_string(path)?;
    toml::from_str(&contents).map_err(ConfigFileError::from)
}

fn default_scheme() -> String {
    DEFAULT_SCHEME.to_string()
}

fn default_tls() -> bool {
    false
}

fn default_non_interactive() -> bool {
    false
}

fn default_quiet() -> bool {
    false
}

fn default_path_prefix() -> String {
    "/api".to_string()
}
