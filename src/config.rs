use crate::constants::{
    DEFAULT_CONFIG_SECTION_NAME, DEFAULT_HOST, DEFAULT_HTTPS_PORT, DEFAULT_HTTP_PORT,
    DEFAULT_PASSWORD, DEFAULT_PATH_PREFIX, DEFAULT_SCHEME, DEFAULT_USERNAME, DEFAULT_VHOST,
    HTTPS_SCHEME,
};
use clap::ArgMatches;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum ConfigFileError {
    #[error("provided config file does not exist")]
    MissingFile(PathBuf),
    #[error("provided configuration section (--node) was not found in the configuration file")]
    MissingConfigSection(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    DeserializationError(#[from] toml::de::Error),
}

type ConfigurationMap<'a> = HashMap<String, SharedSettings>;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SharedSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_uri: Option<String>,

    #[serde(default = "default_tls")]
    pub tls: bool,

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
}

impl SharedSettings {
    pub fn from_config_file(
        path: &PathBuf,
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
        let should_use_tls =
            *cli_args.get_one::<bool>("tls").unwrap() || config_file_defaults.tls || false;
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
            .or_else(|| {
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

        Self {
            tls: should_use_tls,
            base_uri: None,
            scheme: scheme.to_string(),
            hostname: Some(hostname),
            port: Some(port),
            path_prefix: path_prefix.clone(),
            username: Some(username.to_string()),
            password: Some(password.to_string()),
            virtual_host: Some(vhost),
        }
    }

    pub fn new(cli_args: &ArgMatches) -> Self {
        let default_hostname = DEFAULT_HOST.to_string();
        let should_use_tls = cli_args
            .get_one::<bool>("tls")
            .or(Some(&false))
            .cloned()
            .unwrap();
        let scheme = if should_use_tls { "https" } else { "http" };
        let hostname = cli_args
            .get_one::<String>("host")
            .cloned()
            .unwrap_or(default_hostname);
        let port: u16 = cli_args
            .get_one::<u16>("port")
            .cloned()
            .or_else(|| {
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

        Self {
            tls: should_use_tls,
            base_uri: None,
            scheme: scheme.to_string(),
            hostname: Some(hostname),
            port: Some(port),
            path_prefix: path_prefix.clone(),
            username: Some(username.to_string()),
            password: Some(password.to_string()),
            virtual_host: Some(vhost),
        }
    }

    pub fn new_from_uri_with_defaults(
        url: &Url,
        cli_args: &ArgMatches,
        config_file_defaults: &Self,
    ) -> Self {
        let should_use_tls = *cli_args.get_one::<bool>("tls").unwrap()
            || config_file_defaults.tls
            || url.scheme() == HTTPS_SCHEME;

        let scheme = url.scheme().to_string();
        let hostname = url.host_str().unwrap_or(DEFAULT_HOST).to_string();
        let port = url
            .port()
            .or_else(|| {
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

        Self {
            tls: should_use_tls,
            base_uri: Some(url.to_string()),
            scheme: scheme.to_string(),
            hostname: Some(hostname),
            port: Some(port),
            path_prefix,
            username: Some(username),
            password: Some(password),
            virtual_host: Some(vhost),
        }
    }

    pub fn new_from_uri(url: &Url, cli_args: &ArgMatches) -> Self {
        let should_use_tls =
            *cli_args.get_one::<bool>("tls").unwrap() || url.scheme() == HTTPS_SCHEME;

        let scheme = url.scheme().to_string();
        let hostname = url.host_str().unwrap_or(DEFAULT_HOST).to_string();
        let port = url
            .port()
            .or_else(|| {
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

        Self {
            tls: should_use_tls,
            base_uri: Some(url.to_string()),
            scheme: scheme.to_string(),
            hostname: Some(hostname),
            port: Some(port),
            path_prefix,
            username: Some(username),
            password: Some(password),
            virtual_host: Some(vhost),
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

fn from_local_path(path: &PathBuf) -> Result<ConfigurationMap, ConfigFileError> {
    if path.exists() {
        read_from_local_path(path)
    } else {
        Err(ConfigFileError::MissingFile((*path).clone()))
    }
}

fn read_from_local_path(path: &PathBuf) -> Result<ConfigurationMap, ConfigFileError> {
    let contents = std::fs::read_to_string(path)?;
    toml::from_str(&contents)
        .and_then(|t| Ok(t))
        .map_err(|e| ConfigFileError::from(e))
}

fn default_scheme() -> String {
    DEFAULT_SCHEME.to_string()
}

fn default_tls() -> bool {
    false
}

fn default_path_prefix() -> String {
    "/api".to_string()
}
