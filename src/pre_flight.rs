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

use std::env;

/// Represents the two modes of operation for the `rabbitmqadmin` CLI:
/// interactive (driven by a human) and non-interactive (driven by automation tools).
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractivityMode {
    #[default]
    Interactive,
    NonInteractive,
}


impl InteractivityMode {
    pub fn from_env() -> Self {
        if is_enabled_in_env("RABBITMQADMIN_NON_INTERACTIVE_MODE") {
            Self::NonInteractive
        } else {
            Self::Interactive
        }
    }

    pub fn is_non_interactive(&self) -> bool {
        matches!(self, Self::NonInteractive)
    }
}

pub fn is_non_interactive() -> bool {
    InteractivityMode::from_env().is_non_interactive()
}

pub fn should_infer_subcommands() -> bool {
    is_enabled_in_env("RABBITMQADMIN_INFER_SUBCOMMANDS")
}

pub fn should_infer_long_options() -> bool {
    is_enabled_in_env("RABBITMQADMIN_INFER_LONG_OPTIONS")
}

fn is_enabled_in_env(key: &str) -> bool {
    match env::var(key) {
        Ok(val) => val.to_lowercase().trim() == "true",
        Err(_) => false,
    }
}
