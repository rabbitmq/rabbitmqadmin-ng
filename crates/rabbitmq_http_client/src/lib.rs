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
        let response = self.http_get("nodes")?;
        response.json::<Vec<responses::ClusterNode>>()
    }

    pub fn list_vhosts(&self) -> responses::Result<Vec<responses::VirtualHost>> {
        let response = self.http_get("vhosts")?;
        response.json::<Vec<responses::VirtualHost>>()
    }

    pub fn list_users(&self) -> responses::Result<Vec<responses::User>> {
        let response = self.http_get("users")?;
        response.json::<Vec<responses::User>>()
    }

    pub fn get_node_info(&self, name: &str) -> responses::Result<responses::ClusterNode> {
        let response = self.http_get(&format!("nodes/{}", name))?;
        let node = response.json::<responses::ClusterNode>()?;
        Ok(node)
    }

    pub fn get_vhost(&self, name: &str) -> responses::Result<responses::VirtualHost> {
        let response = self.http_get(&format!("vhosts/{}", name))?;
        let node = response.json::<responses::VirtualHost>()?;
        Ok(node)
    }

    pub fn get_user(&self, name: &str) -> responses::Result<responses::User> {
        let response = self.http_get(&format!("users/{}", name))?;
        let node = response.json::<responses::User>()?;
        Ok(node)
    }

    //
    // Implementation
    //

    fn http_get(&self, path: &str) -> reqwest::Result<reqwest::blocking::Response> {
        HttpClient::new()
            .get(self.rooted_path(path))
            .basic_auth(self.username, self.password)
            .send()
    }

    fn rooted_path(&self, path: &str) -> String {
        return format!("{}/{}", self.endpoint, path)
    }
}