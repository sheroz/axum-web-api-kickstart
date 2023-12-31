pub const TEST_ADMIN_USERNAME: &str = "admin";
pub const TEST_ADMIN_PASSWORD_HASH: &str = "7c44575b741f02d49c3e988ba7aa95a8fb6d90c0ef63a97236fa54bfcfbd9d51";

type GenericResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub mod auth;
pub mod fetch;
pub mod route;
pub mod test_config;
pub mod users;

