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

use rabbitmqadmin::pre_flight::InteractivityMode;
use std::env;

#[test]
fn test_interactivity_mode_default() {
    let mode = InteractivityMode::default();
    assert_eq!(mode, InteractivityMode::Interactive);
    assert!(!mode.is_non_interactive());
}

#[test]
fn test_interactivity_mode_non_interactive() {
    let mode = InteractivityMode::NonInteractive;
    assert!(mode.is_non_interactive());
}

#[test]
fn test_interactivity_mode_from_env_interactive() {
    // Clear the environment variable to test the default case
    unsafe {
        env::remove_var("RABBITMQADMIN_NON_INTERACTIVE_MODE");
    }
    let mode = InteractivityMode::from_env();
    assert_eq!(mode, InteractivityMode::Interactive);
}

#[test]
fn test_interactivity_mode_from_env_non_interactive() {
    // Set the environment variable
    unsafe {
        env::set_var("RABBITMQADMIN_NON_INTERACTIVE_MODE", "true");
    }
    let mode = InteractivityMode::from_env();
    assert_eq!(mode, InteractivityMode::NonInteractive);

    // Clean up
    unsafe {
        env::remove_var("RABBITMQADMIN_NON_INTERACTIVE_MODE");
    }
}
