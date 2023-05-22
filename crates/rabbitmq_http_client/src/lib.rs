pub mod requests;
pub mod responses;

use reqwest::blocking::Client as HttpClient;

pub struct Client<'a> {
    endpoint: &'a str,
    username: &'a str,
    password: Option<&'a str>
}

impl<'a> Client<'a> {
    pub fn new_with_basic_auth_credentials(
        endpoint: &'a str,
        username: &'a str,
        password: Option<&'a str>) -> Self {
        Self {
            endpoint: endpoint,
            username: username,
            password: password
        }
    }

    pub fn list_nodes(&self) -> responses::Result<Vec<responses::ClusterNode>> {
        let response = HttpClient::new()
            .get(self.rooted_path("nodes/"))
            .basic_auth(self.username, self.password)
            .send()?;

        println!("HTTP API response: {:?}", response);
        response.json::<Vec<responses::ClusterNode>>()
    }

    pub fn list_vhosts(&self) -> responses::Result<Vec<responses::VirtualHost>> {
        let response = HttpClient::new()
            .get(self.rooted_path("vhosts/"))
            .basic_auth(self.username, self.password)
            .send()?;

        println!("HTTP API response: {:?}", response);
        response.json::<Vec<responses::VirtualHost>>()
    }

    pub fn list_users(&self) -> responses::Result<Vec<responses::User>> {
        let response = HttpClient::new()
            .get(self.rooted_path("users/"))
            .basic_auth(self.username, self.password)
            .send()?;

        println!("HTTP API response: {:?}", response);
        response.json::<Vec<responses::User>>()
    }

    pub fn get_node_info(&self, name: &str) -> responses::Result<responses::ClusterNode> {
        let response = HttpClient::new()
            .get(self.rooted_path(&format!("vhosts/{}", name)))
            .basic_auth(self.username, self.password)
            .send()?;

        println!("response: {:?}", response);
        let node = response.json::<responses::ClusterNode>()?;
        Ok(node)
    }

    //
    // Implementation
    //

    fn rooted_path(&self, path: &str) -> String {
        return format!("{}/{}", self.endpoint, path)
    }
}