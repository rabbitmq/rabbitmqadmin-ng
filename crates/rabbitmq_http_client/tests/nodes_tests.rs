use rabbitmq_http_client::blocking::Client;

mod common;
use crate::common::{endpoint, USERNAME, PASSWORD};

#[test]
fn test_list_nodes() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, Some(PASSWORD));
    let result = rc.list_nodes();

    assert!(result.is_ok());
    let vec = result.unwrap();
    assert!(vec.iter().find(|n| n.name.starts_with("rabbit@")).is_some())
}

#[test]
fn test_get_node_info() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, Some(PASSWORD));
    let nodes = rc.list_nodes().unwrap();
    let name = nodes.first().unwrap().name.clone();
    let node = &rc.get_node_info(&name).unwrap();

    assert!(node.processors >= 1);
    assert!(node.uptime >= 1);
    assert!(node.total_erlang_processes >= 1);
}