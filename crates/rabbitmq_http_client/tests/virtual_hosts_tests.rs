use rabbitmq_http_client::blocking::Client;

mod common;
use crate::common::{endpoint, USERNAME, PASSWORD};

#[test]
fn test_list_vhosts() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, Some(PASSWORD));
    let result = rc.list_vhosts();
    
    assert!(result.is_ok());
    let vec = result.unwrap();
    assert!(vec.iter().find(|vh| vh.name == "/").is_some())
}

#[test]
fn test_get_vhost() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, Some(PASSWORD));
    let name = "/";
    let result = rc.get_vhost(name);

    assert!(result.is_ok());
    let vh = result.unwrap();
    assert!(vh.name == name);
}