mod requests;
mod responses;

use reqwest::RequestBuilder;

struct Client<'a> {
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

    pub async fn get_node_info(&self, name: &str) -> responses::Result<responses::ClusterNode> {
        let response = reqwest::Client::new()
            .get(self.rooted_path(&format!("/nodes/{}", name)))
            .basic_auth(self.username, self.password)
            .send().await?;

        println!("response: {:?}", response);
        let node = response.json::<responses::ClusterNode>().await?;
        Ok(node)
    }

    //
    // Implementation
    //

    fn rooted_path(&self, path: &str) -> String {
        return format!("{}/{}", self.endpoint, path)
    }
}