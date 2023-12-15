use std::{net::SocketAddr, str::FromStr};

#[derive(Debug)]
pub struct Config {
    // service
    pub service_host: String,
    pub service_port: u16,

    // redis
    pub redis_host: String,
    pub redis_port: u16,

    // postgres
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_db: String,
    pub postgres_connection_pool: u32
}

impl Config {
    pub fn service_http_addr(&self) -> String {
        format!("http://{}:{}", self.service_host, self.service_port)
    }

    pub fn service_socket_addr(&self) -> SocketAddr {
        SocketAddr::from_str(&format!("{}:{}", self.service_host, self.service_port)).unwrap()
    }

    pub fn redis_url(&self) -> String {
        format!("redis://{}:{}", self.redis_host, self.redis_port)
    }

    pub fn postgres_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.postgres_user,
            self.postgres_password,
            self.postgres_host,
            self.postgres_port,
            self.postgres_db
        )
    }
}
pub fn from_dotenv() -> Config {
    // load .env file
    dotenv::dotenv().expect("Failed to load .env file");

    // parse configuration
    let config = Config {
        service_host: env_get("SERVICE_HOST"),
        service_port: env_parse("SERVICE_PORT"),
        redis_host: env_get("REDIS_HOST"),
        redis_port: env_parse("REDIS_PORT"),
        postgres_user: env_get("POSTGRES_USER"),
        postgres_password: env_get("POSTGRES_PASSWORD"),
        postgres_host: env_get("POSTGRES_HOST"),
        postgres_port: env_parse("POSTGRES_PORT"),
        postgres_db: env_get("POSTGRES_DB"),
        postgres_connection_pool: env_parse("POSTGRES_CONNECTION_POOL"),
    };

    tracing::trace!("configuration: {:#?}", config);
    config
}

#[inline]
fn env_get(key: &str) -> String {
    match std::env::var(key) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("{} {}", key, e);
            tracing::error!(msg);
            panic!("{msg}");
        }
    }
}

#[inline]
fn env_parse<T: std::str::FromStr>(key: &str) -> T {
    match env_get(key).parse() {
        Ok(v) => v,
        Err(_) => {
            let msg = format!("Failed to parse: {}", key);
            tracing::error!(msg);
            panic!("{msg}");
        }
    }
}
