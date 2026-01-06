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

use tabled::builder::Builder;
use tabled::{Table, Tabled};

pub fn parse_columns(columns_arg: &str) -> Vec<String> {
    columns_arg
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn build_table_with_columns<T: Tabled>(data: &[T], columns: &[String]) -> Table {
    let mut builder = Builder::default();

    let headers: Vec<String> = T::headers()
        .into_iter()
        .map(|c| c.to_string().to_lowercase())
        .collect();

    let valid_columns: Vec<(usize, &String)> = columns
        .iter()
        .filter_map(|col| headers.iter().position(|h| h == col).map(|idx| (idx, col)))
        .collect();

    builder.push_record(valid_columns.iter().map(|(_, col)| col.as_str()));

    for item in data {
        let fields: Vec<String> = item.fields().into_iter().map(|c| c.to_string()).collect();

        let row: Vec<&str> = valid_columns
            .iter()
            .map(|(idx, _)| fields[*idx].as_str())
            .collect();
        builder.push_record(row);
    }

    builder.build()
}
