use clap::ArgMatches;

use rabbitmq_http_client::responses;
use rabbitmq_http_client::responses::Result as ClientResult;
use rabbitmq_http_client::Client as APIClient;

use crate::cli::SharedFlags;

pub fn list_nodes(general_args: &ArgMatches) -> ClientResult<Vec<responses::ClusterNode>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc =
        APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, Some(&sf.password));
    rc.list_nodes()
}

pub fn list_vhosts(general_args: &ArgMatches) -> ClientResult<Vec<responses::VirtualHost>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc =
        APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, Some(&sf.password));
    rc.list_vhosts()
}

pub fn list_users(general_args: &ArgMatches) -> ClientResult<Vec<responses::User>> {
    let sf = SharedFlags::from_args(general_args);
    let endpoint = sf.endpoint();
    let rc =
        APIClient::new_with_basic_auth_credentials(&endpoint, &sf.username, Some(&sf.password));
    rc.list_users()
}
