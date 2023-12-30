use core::fmt;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::{net::SocketAddr, sync::OnceLock};

pub static CONFIG: OnceLock<Config> = OnceLock::new();

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
    pub postgres_connection_pool: u32,

    // JWT
    pub jwt_secret: String,
    pub jwt_keys: JwtKeys,
    pub jwt_expire_access_token_seconds: i64, 
    pub jwt_expire_refresh_token_seconds: i64, 
    pub jwt_use_revoked_list: bool
}
pub struct JwtKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

// a blank impl fmt::Debug for JwtKeys
// there is no debug(skip) option for #[derive(Debug)] currently in Rust 1.74
impl fmt::Debug for JwtKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JwtKeys").finish()
    }
}

impl JwtKeys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

impl Config {
    pub fn service_http_addr(&self) -> String {
        format!("{}://{}:{}", "http", self.service_host, self.service_port)
    }

    pub fn service_socket_addr(&self) -> SocketAddr {
        use std::str::FromStr;
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

pub fn load_from_dotenv() {
    // load .env file
    dotenv::dotenv().expect("Failed to load .env file");

    let jwt_secret = env_get("JWT_SECRET");

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
        jwt_keys: JwtKeys::new(jwt_secret.as_bytes()),
        jwt_secret,
        jwt_expire_access_token_seconds: env_parse("JWT_EXPIRE_ACCESS_TOKEN_SECONDS"),
        jwt_expire_refresh_token_seconds: env_parse("JWT_EXPIRE_REFRESH_TOKEN_SECONDS"),
        jwt_use_revoked_list: env_parse("JWT_USE_REVOKED_LIST")
    };

    tracing::trace!("configuration: {:#?}", config);
    CONFIG.get_or_init(|| config);
}

pub fn get() -> &'static Config {
    CONFIG.get().unwrap()
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
