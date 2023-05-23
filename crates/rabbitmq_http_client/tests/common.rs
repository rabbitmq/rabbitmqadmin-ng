pub const ENDPOINT: &str = "http://localhost:15672/api";
pub const USERNAME: &str = "guest";
pub const PASSWORD: &str = "guest";

pub fn endpoint() -> String {
    ENDPOINT.to_owned()
}