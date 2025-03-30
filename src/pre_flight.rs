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

pub fn is_non_interactive() -> bool {
    is_enabled_in_env("RABBITMQADMIN_NON_INTERACTIVE_MODE")
}

pub fn should_infer_subcommands() -> bool {
    is_enabled_in_env("RABBITMQADMIN_INFER_SUBCOMMANDS")
}

pub fn should_infer_long_options() -> bool {
    is_enabled_in_env("RABBITMQADMIN_INFER_LONG_OPTIONS")
}

fn is_enabled_in_env(key: &str) -> bool {
    match std::env::var(key) {
        Ok(val) => val.to_lowercase().trim() == "true",
        Err(_) => false,
    }
}
