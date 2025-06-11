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
pub const DEFAULT_SCHEME: &str = "http";
pub const HTTPS_SCHEME: &str = "https";
pub const DEFAULT_HOST: &str = "localhost";
pub const DEFAULT_PORT_STR: &str = "15672";
pub const DEFAULT_HTTPS_PORT: u16 = 15671;
pub const DEFAULT_HTTP_PORT: u16 = 15672;
// This path prefix that precedes
pub const DEFAULT_PATH_PREFIX: &str = "/api";
pub const DEFAULT_VHOST: &str = "/";

pub const DEFAULT_USERNAME: &str = "guest";
pub const DEFAULT_PASSWORD: &str = "guest";

pub const DEFAULT_QUEUE_TYPE: &str = "classic";

// default node section in the configuration file
pub const DEFAULT_NODE_ALIAS: &str = "default";

// Default local path to rabbitmqadmin.conf
pub const DEFAULT_CONFIG_FILE_PATH: &str = "~/.rabbitmqadmin.conf";
pub const DEFAULT_CONFIG_SECTION_NAME: &str = "default";

pub const TANZU_COMMAND_PREFIX: &str = "tanzu";

pub const DEFAULT_BLANKET_POLICY_PRIORITY: i16 = -20;
