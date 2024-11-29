// Copyright (C) 2023-2024 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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
use std::time::Duration;

#[allow(dead_code)]
pub fn await_metric_emission(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

#[allow(dead_code)]
pub fn await_queue_metric_emission() {
    let delay = env::var("TEST_STATS_DELAY").unwrap_or("500".to_owned());
    await_metric_emission(delay.parse::<u64>().unwrap());
}
