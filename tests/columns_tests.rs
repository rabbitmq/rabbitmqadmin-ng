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

use rabbitmqadmin::columns::parse_columns;

#[test]
fn test_parse_columns_basic() {
    let result = parse_columns("name,type,value");
    assert_eq!(result, vec!["name", "type", "value"]);
}

#[test]
fn test_parse_columns_with_spaces() {
    let result = parse_columns(" name , type , value ");
    assert_eq!(result, vec!["name", "type", "value"]);
}

#[test]
fn test_parse_columns_lowercase() {
    let result = parse_columns("Name,TYPE,VaLuE");
    assert_eq!(result, vec!["name", "type", "value"]);
}

#[test]
fn test_parse_columns_empty() {
    let result = parse_columns("");
    assert!(result.is_empty());
}

#[test]
fn test_parse_columns_with_empty_parts() {
    let result = parse_columns("name,,type");
    assert_eq!(result, vec!["name", "type"]);
}

#[test]
fn test_parse_columns_single() {
    let result = parse_columns("name");
    assert_eq!(result, vec!["name"]);
}

#[test]
fn test_parse_columns_trailing_comma() {
    let result = parse_columns("name,type,");
    assert_eq!(result, vec!["name", "type"]);
}

mod build_table_tests {
    use rabbitmq_http_client::responses::VirtualHost;
    use rabbitmqadmin::columns::build_table_with_columns;

    fn sample_vhosts() -> Vec<VirtualHost> {
        vec![
            VirtualHost {
                name: "/".to_string(),
                tags: None,
                description: Some("Default virtual host".to_string()),
                default_queue_type: Some("classic".to_string()),
                metadata: None,
            },
            VirtualHost {
                name: "production".to_string(),
                tags: None,
                description: Some("Production environment".to_string()),
                default_queue_type: Some("quorum".to_string()),
                metadata: None,
            },
        ]
    }

    #[test]
    fn test_build_table_with_valid_columns() {
        let data = sample_vhosts();
        let columns = vec!["name".to_string(), "description".to_string()];
        let table = build_table_with_columns(&data, &columns);
        let output = table.to_string();
        assert!(output.contains("name"));
        assert!(output.contains("description"));
        assert!(output.contains("/"));
        assert!(output.contains("Default virtual host"));
    }

    #[test]
    fn test_build_table_filters_invalid_columns() {
        let data = sample_vhosts();
        let columns = vec!["name".to_string(), "nonexistent".to_string()];
        let table = build_table_with_columns(&data, &columns);
        let output = table.to_string();
        assert!(output.contains("name"));
        assert!(output.contains("/"));
        assert!(!output.contains("nonexistent"));
    }

    #[test]
    fn test_build_table_with_all_invalid_columns() {
        let data = sample_vhosts();
        let columns = vec!["invalid1".to_string(), "invalid2".to_string()];
        let table = build_table_with_columns(&data, &columns);
        let output = table.to_string();
        assert!(!output.contains("/"));
        assert!(!output.contains("production"));
    }

    #[test]
    fn test_build_table_with_empty_data() {
        let data: Vec<VirtualHost> = vec![];
        let columns = vec!["name".to_string(), "description".to_string()];
        let table = build_table_with_columns(&data, &columns);
        let output = table.to_string();
        assert!(output.contains("name"));
        assert!(output.contains("description"));
    }

    #[test]
    fn test_build_table_with_empty_columns() {
        let data = sample_vhosts();
        let columns: Vec<String> = vec![];
        let table = build_table_with_columns(&data, &columns);
        let output = table.to_string();
        assert!(!output.contains("/"));
    }

    #[test]
    fn test_build_table_preserves_column_order() {
        let data = sample_vhosts();
        let columns = vec!["default_queue_type".to_string(), "name".to_string()];
        let table = build_table_with_columns(&data, &columns);
        let output = table.to_string();
        let type_pos = output.find("default_queue_type").unwrap();
        let name_pos = output.find("name").unwrap();
        assert!(type_pos < name_pos);
    }
}
