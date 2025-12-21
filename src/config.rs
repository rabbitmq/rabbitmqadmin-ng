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
    DEFAULT_PASSWORD, DEFAULT_PATH_PREFIX, DEFAULT_USERNAME, DEFAULT_VHOST,
};
use crate::errors::CommandRunError;
use crate::output::TableStyle;
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{collections::HashMap, fs, io};
use tabled::Tabled;
use thiserror::Error;
use toml_edit::{DocumentMut, Item, Table, Value};
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scheme {
    #[default]
    Http,
    Https,
}

impl Scheme {
    pub fn is_https(&self) -> bool {
        matches!(self, Scheme::Https)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Scheme::Http => "http",
            Scheme::Https => "https",
        }
    }
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Scheme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "http" => Ok(Scheme::Http),
            "https" => Ok(Scheme::Https),
            _ => Err(format!(
                "Invalid scheme: '{}'. Expected 'http' or 'https'",
                s
            )),
        }
    }
}

impl From<&str> for Scheme {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or_default()
    }
}

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
    #[error("the provided config file at '{0}' does not exist")]
    MissingFile(PathBuf),
    #[error(
        "specified configuration section (--node) '{0}' was not found in the configuration file"
    )]
    MissingConfigSection(String),
    #[error("node '{0}' already exists in the configuration file. Use 'update_node' to modify it")]
    NodeAlreadyExists(String),
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("failed to deserialize the config file. Make sure it is valid TOML. Details: {0}")]
    DeserializationError(#[from] toml::de::Error),
    #[error("failed to parse the config file. Make sure it is valid TOML. Details: {0}")]
    ParseError(#[from] toml_edit::TomlError),
}

type ConfigurationMap<'a> = HashMap<String, SharedSettings>;

/// Represents a set of settings that can be set both via
/// the command line arguments and an optional configuration file.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SharedSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_uri: Option<String>,

    #[serde(default = "default_tls")]
    pub tls: bool,

    #[serde(default = "default_non_interactive")]
    pub non_interactive: bool,
    #[serde(default = "default_quiet")]
    pub quiet: bool,

    #[serde(default)]
    pub scheme: Scheme,
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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_certificate_bundle_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_certificate_file_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_private_key_file_path: Option<PathBuf>,
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

    pub fn from_args_with_defaults(
        general_args: &ArgMatches,
        config_file_defaults: &Self,
    ) -> Result<Self, CommandRunError> {
        let base_uri = general_args
            .get_one::<String>("base_uri")
            .cloned()
            .or_else(|| config_file_defaults.base_uri.clone());

        if let Some(s) = base_uri {
            let url = Url::parse(&s).map_err(|e| CommandRunError::InvalidBaseUri {
                uri: s.clone(),
                message: e.to_string(),
            })?;
            Ok(SharedSettings::new_from_uri_with_defaults(
                &url,
                general_args,
                config_file_defaults,
            ))
        } else {
            Ok(SharedSettings::new_with_defaults(
                general_args,
                config_file_defaults,
            ))
        }
    }

    pub fn from_args(general_args: &ArgMatches) -> Result<Self, CommandRunError> {
        let base_uri = general_args.get_one::<String>("base_uri").cloned();

        if let Some(s) = base_uri {
            let url = Url::parse(&s).map_err(|e| CommandRunError::InvalidBaseUri {
                uri: s.clone(),
                message: e.to_string(),
            })?;
            Ok(SharedSettings::new_from_uri(&url, general_args))
        } else {
            Ok(SharedSettings::new(general_args))
        }
    }

    pub fn new_with_defaults(cli_args: &ArgMatches, config_file_defaults: &Self) -> Self {
        let default_hostname = DEFAULT_HOST.to_string();
        let should_use_tls =
            cli_args.get_one::<bool>("tls").cloned().unwrap_or(false) || config_file_defaults.tls;

        let non_interactive = cli_args
            .get_one::<bool>("non_interactive")
            .cloned()
            .unwrap_or(false)
            || config_file_defaults.non_interactive;
        let quiet = cli_args.get_one::<bool>("quiet").cloned().unwrap_or(false)
            || config_file_defaults.quiet;
        let scheme = if should_use_tls {
            Scheme::Https
        } else {
            config_file_defaults.scheme
        };
        let hostname = cli_args
            .get_one::<String>("host")
            .cloned()
            .or_else(|| config_file_defaults.hostname.clone())
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
            .or_else(|| config_file_defaults.username.clone())
            .unwrap_or(DEFAULT_USERNAME.to_string());
        let password = cli_args
            .get_one::<String>("password")
            .cloned()
            .or_else(|| config_file_defaults.password.clone())
            .unwrap_or(DEFAULT_PASSWORD.to_string());
        let vhost = cli_args
            .get_one::<String>("vhost")
            .cloned()
            .or_else(|| config_file_defaults.virtual_host.clone())
            .unwrap_or(DEFAULT_VHOST.to_owned());
        let table_style = cli_args
            .get_one::<TableStyle>("table_style")
            .cloned()
            .or(Some(TableStyle::default()))
            .unwrap_or_default();

        let ca_certificate_bundle_path = cli_args
            .get_one::<PathBuf>("ca_certificate_bundle_path")
            .cloned()
            .or_else(|| config_file_defaults.ca_certificate_bundle_path.clone());

        let client_certificate_file_path = cli_args
            .get_one::<PathBuf>("client_certificate_file_path")
            .cloned()
            .or_else(|| config_file_defaults.client_certificate_file_path.clone());

        let client_private_key_file_path = cli_args
            .get_one::<PathBuf>("client_private_key_file_path")
            .cloned()
            .or_else(|| config_file_defaults.client_private_key_file_path.clone());

        Self {
            tls: should_use_tls,
            ca_certificate_bundle_path,
            client_certificate_file_path,
            client_private_key_file_path,

            non_interactive,
            quiet,
            base_uri: None,
            scheme,
            hostname: Some(hostname),
            port: Some(port),
            path_prefix,
            username: Some(username),
            password: Some(password),
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
        let scheme = if should_use_tls {
            Scheme::Https
        } else {
            Scheme::default()
        };
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

        let ca_certificate_bundle_path = cli_args
            .get_one::<PathBuf>("ca_certificate_bundle_path")
            .cloned();

        let client_certificate_file_path = cli_args
            .get_one::<PathBuf>("client_certificate_file_path")
            .cloned();

        let client_private_key_file_path = cli_args
            .get_one::<PathBuf>("client_private_key_file_path")
            .cloned();

        Self {
            tls: should_use_tls,
            ca_certificate_bundle_path,
            client_certificate_file_path,
            client_private_key_file_path,

            non_interactive,
            quiet,
            base_uri: None,
            scheme,
            hostname: Some(hostname),
            port: Some(port),
            path_prefix,
            username: Some(username),
            password: Some(password),
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
            || url.scheme() == "https";
        let non_interactive = cli_args
            .get_one::<bool>("non_interactive")
            .cloned()
            .unwrap_or(false)
            || config_file_defaults.non_interactive;
        let quiet = cli_args.get_one::<bool>("quiet").cloned().unwrap_or(false)
            || config_file_defaults.quiet;

        let scheme = if should_use_tls {
            Scheme::Https
        } else {
            config_file_defaults.scheme
        };
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
            .or_else(|| config_file_defaults.username.clone())
            .unwrap_or(DEFAULT_USERNAME.to_string());
        let password = cli_args
            .get_one::<String>("password")
            .cloned()
            .or_else(|| config_file_defaults.password.clone())
            .unwrap_or(DEFAULT_PASSWORD.to_string());
        let vhost = cli_args
            .get_one::<String>("vhost")
            .cloned()
            .or_else(|| config_file_defaults.virtual_host.clone())
            .unwrap_or(DEFAULT_VHOST.to_owned());
        let table_style = cli_args
            .get_one::<TableStyle>("table_style")
            .cloned()
            .or(Some(TableStyle::default()))
            .unwrap_or_default();

        let ca_certificate_bundle_path = cli_args
            .get_one::<PathBuf>("ca_certificate_bundle_path")
            .cloned();

        let client_certificate_file_path = cli_args
            .get_one::<PathBuf>("client_certificate_file_path")
            .cloned();

        let client_private_key_file_path = cli_args
            .get_one::<PathBuf>("client_private_key_file_path")
            .cloned();

        Self {
            tls: should_use_tls,
            ca_certificate_bundle_path,
            client_certificate_file_path,
            client_private_key_file_path,

            non_interactive,
            quiet,
            base_uri: Some(url.to_string()),
            scheme,
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
            cli_args.get_one::<bool>("tls").cloned().unwrap_or(false) || url.scheme() == "https";
        let non_interactive = cli_args
            .get_one::<bool>("non_interactive")
            .cloned()
            .unwrap_or(default_non_interactive());
        let quiet = cli_args
            .get_one::<bool>("quiet")
            .cloned()
            .unwrap_or(default_quiet());

        let scheme = if should_use_tls {
            Scheme::Https
        } else {
            Scheme::from(url.scheme())
        };
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

        let ca_certificate_bundle_path = cli_args
            .get_one::<PathBuf>("ca_certificate_bundle_path")
            .cloned();

        let client_certificate_file_path = cli_args
            .get_one::<PathBuf>("client_certificate_file_path")
            .cloned();

        let client_private_key_file_path = cli_args
            .get_one::<PathBuf>("client_private_key_file_path")
            .cloned();

        Self {
            tls: should_use_tls,
            ca_certificate_bundle_path,
            client_certificate_file_path,
            client_private_key_file_path,

            non_interactive,
            quiet,
            base_uri: Some(url.to_string()),
            scheme,
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
        let prefix = if self.path_prefix.starts_with('/') {
            self.path_prefix.clone()
        } else {
            format!("/{}", self.path_prefix)
        };
        format!(
            "{}://{}:{}{}",
            self.scheme,
            self.hostname.as_ref().unwrap(),
            self.port.unwrap(),
            prefix
        )
        .trim()
        .to_string()
    }
}

fn from_local_path(path: &Path) -> Result<ConfigurationMap<'_>, ConfigFileError> {
    let expanded_s = shellexpand::tilde(&path.to_string_lossy()).to_string();
    let expanded_path = PathBuf::from(&expanded_s);
    if expanded_path.exists() {
        read_from_local_path(&expanded_path)
    } else {
        Err(ConfigFileError::MissingFile((*expanded_path).to_path_buf()))
    }
}

fn read_from_local_path(path: &PathBuf) -> Result<ConfigurationMap<'_>, ConfigFileError> {
    let contents = fs::read_to_string(path)?;
    toml::from_str(&contents).map_err(ConfigFileError::from)
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

const PASSWORD_MASK: &str = "********";

#[derive(Debug, Clone, Tabled)]
pub struct ConfigPathEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Tabled)]
pub struct NodeConfigEntry {
    pub name: String,
    pub hostname: String,
    pub port: String,
    pub scheme: String,
    pub base_uri: String,
    pub username: String,
    pub password: String,
    pub vhost: String,
    pub path_prefix: String,
}

impl NodeConfigEntry {
    pub fn from_settings_with_name(
        name: &str,
        settings: &SharedSettings,
        reveal_password: bool,
    ) -> Self {
        let password = if reveal_password {
            settings.password.clone().unwrap_or_default()
        } else {
            PASSWORD_MASK.to_string()
        };
        Self {
            name: name.to_string(),
            hostname: settings.hostname.clone().unwrap_or_default(),
            port: settings.port.map(|p| p.to_string()).unwrap_or_default(),
            scheme: if settings.scheme == Scheme::Http {
                String::new()
            } else {
                settings.scheme.to_string()
            },
            base_uri: settings.base_uri.clone().unwrap_or_default(),
            username: settings.username.clone().unwrap_or_default(),
            password,
            vhost: settings.virtual_host.clone().unwrap_or_default(),
            path_prefix: if settings.path_prefix == DEFAULT_PATH_PREFIX {
                String::new()
            } else {
                settings.path_prefix.clone()
            },
        }
    }
}

pub fn resolve_config_file_path(cli_path: Option<&PathBuf>) -> PathBuf {
    let path = cli_path
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| crate::constants::DEFAULT_CONFIG_FILE_PATH.to_string());
    let expanded = shellexpand::tilde(&path).to_string();
    let path_buf = PathBuf::from(&expanded);
    path_buf.canonicalize().unwrap_or(path_buf)
}

pub fn config_file_exists(path: &Path) -> bool {
    let expanded_s = shellexpand::tilde(&path.to_string_lossy()).to_string();
    let expanded_path = PathBuf::from(&expanded_s);
    expanded_path.exists()
}

pub fn list_all_nodes(path: &Path) -> Result<Vec<(String, SharedSettings)>, ConfigFileError> {
    let cm = from_local_path(path)?;
    Ok(cm.into_iter().collect())
}

fn load_config_document(
    path: &Path,
    create_if_missing: bool,
) -> Result<(PathBuf, DocumentMut), ConfigFileError> {
    let expanded_s = shellexpand::tilde(&path.to_string_lossy()).to_string();
    let expanded_path = PathBuf::from(&expanded_s);

    let contents = if expanded_path.exists() {
        fs::read_to_string(&expanded_path)?
    } else if create_if_missing {
        if let Some(parent) = expanded_path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }
        String::new()
    } else {
        return Err(ConfigFileError::MissingFile(expanded_path));
    };

    let doc = contents.parse::<DocumentMut>()?;
    Ok((expanded_path, doc))
}

pub fn add_node_to_config_file(
    path: &Path,
    node_name: &str,
    settings: &SharedSettings,
    create_file_if_missing: bool,
) -> Result<(), ConfigFileError> {
    let (expanded_path, mut doc) = load_config_document(path, create_file_if_missing)?;

    if doc.contains_key(node_name) {
        return Err(ConfigFileError::NodeAlreadyExists(node_name.to_string()));
    }

    let node_table = build_node_table(settings);
    doc.insert(node_name, Item::Table(node_table));

    fs::write(&expanded_path, doc.to_string())?;
    Ok(())
}

pub fn update_node_in_config_file(
    path: &Path,
    node_name: &str,
    settings: &SharedSettings,
    create_file_if_missing: bool,
) -> Result<(), ConfigFileError> {
    let (expanded_path, mut doc) = load_config_document(path, create_file_if_missing)?;

    // Get existing node table or create a new one
    let existing_table = doc.get_mut(node_name).and_then(|item| item.as_table_mut());

    if let Some(table) = existing_table {
        // Merge new settings into existing table
        merge_settings_into_table(table, settings);
    } else {
        // Node doesn't exist, create new table
        let node_table = build_node_table(settings);
        doc.insert(node_name, Item::Table(node_table));
    }

    fs::write(&expanded_path, doc.to_string())?;
    Ok(())
}

fn merge_settings_into_table(table: &mut Table, settings: &SharedSettings) {
    if let Some(ref base_uri) = settings.base_uri {
        table.insert("base_uri", Value::from(base_uri.as_str()).into());
    }
    if let Some(ref hostname) = settings.hostname {
        table.insert("hostname", Value::from(hostname.as_str()).into());
    }
    if let Some(port) = settings.port {
        table.insert("port", Value::from(port as i64).into());
    }
    if let Some(ref username) = settings.username {
        table.insert("username", Value::from(username.as_str()).into());
    }
    if let Some(ref password) = settings.password {
        table.insert("password", Value::from(password.as_str()).into());
    }
    if let Some(ref vhost) = settings.virtual_host {
        table.insert("virtual_host", Value::from(vhost.as_str()).into());
    }
    if settings.scheme != Scheme::Http {
        table.insert("scheme", Value::from(settings.scheme.as_str()).into());
    }
    if !settings.path_prefix.is_empty() {
        table.insert(
            "path_prefix",
            Value::from(settings.path_prefix.as_str()).into(),
        );
    }
    if settings.tls {
        table.insert("tls", Value::from(true).into());
    }
    if let Some(ref path) = settings.ca_certificate_bundle_path {
        table.insert(
            "ca_certificate_bundle_path",
            Value::from(path.to_string_lossy().as_ref()).into(),
        );
    }
    if let Some(ref path) = settings.client_certificate_file_path {
        table.insert(
            "client_certificate_file_path",
            Value::from(path.to_string_lossy().as_ref()).into(),
        );
    }
    if let Some(ref path) = settings.client_private_key_file_path {
        table.insert(
            "client_private_key_file_path",
            Value::from(path.to_string_lossy().as_ref()).into(),
        );
    }
}

fn build_node_table(settings: &SharedSettings) -> Table {
    let mut node_table = Table::new();
    if let Some(ref base_uri) = settings.base_uri {
        node_table.insert("base_uri", Value::from(base_uri.as_str()).into());
    }
    if let Some(ref hostname) = settings.hostname {
        node_table.insert("hostname", Value::from(hostname.as_str()).into());
    }
    if let Some(port) = settings.port {
        node_table.insert("port", Value::from(port as i64).into());
    }
    if let Some(ref username) = settings.username {
        node_table.insert("username", Value::from(username.as_str()).into());
    }
    if let Some(ref password) = settings.password {
        node_table.insert("password", Value::from(password.as_str()).into());
    }
    if let Some(ref vhost) = settings.virtual_host {
        node_table.insert("virtual_host", Value::from(vhost.as_str()).into());
    }
    if settings.scheme != Scheme::Http {
        node_table.insert("scheme", Value::from(settings.scheme.as_str()).into());
    }
    if !settings.path_prefix.is_empty() && settings.path_prefix != DEFAULT_PATH_PREFIX {
        node_table.insert(
            "path_prefix",
            Value::from(settings.path_prefix.as_str()).into(),
        );
    }
    if settings.tls {
        node_table.insert("tls", Value::from(true).into());
    }
    if let Some(ref path) = settings.ca_certificate_bundle_path {
        node_table.insert(
            "ca_certificate_bundle_path",
            Value::from(path.to_string_lossy().as_ref()).into(),
        );
    }
    if let Some(ref path) = settings.client_certificate_file_path {
        node_table.insert(
            "client_certificate_file_path",
            Value::from(path.to_string_lossy().as_ref()).into(),
        );
    }
    if let Some(ref path) = settings.client_private_key_file_path {
        node_table.insert(
            "client_private_key_file_path",
            Value::from(path.to_string_lossy().as_ref()).into(),
        );
    }
    node_table
}

pub fn delete_node_from_config_file(
    path: &Path,
    node_name: &str,
    create_file_if_missing: bool,
) -> Result<(), ConfigFileError> {
    let (expanded_path, mut doc) = load_config_document(path, create_file_if_missing)?;

    doc.remove(node_name);

    fs::write(&expanded_path, doc.to_string())?;
    Ok(())
}
