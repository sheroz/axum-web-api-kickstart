// using macro to reduce boilerplate
macro_rules! env_get {
    ($key:expr) => {
        match std::env::var($key) {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("{} {}", $key, e);
                tracing::error!(msg);
                panic!("{msg}");
            }
        }
    };
}

macro_rules! env_parse {
    ($key:expr) => {
        match env_get!($key).parse() {
            Ok(v) => v,
            Err(_) => {
                let msg = format!("Failed to parse: {}", $key);
                tracing::error!(msg);
                panic!("{msg}");
            }
        }
    };
}

pub struct Config {
    // redis
    pub redis_host: String,
    pub redis_port: u16,

    // postgres
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_db: String,
}
impl Config {
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
    Config {
        redis_host: env_get!("REDIS_HOST"),
        redis_port: env_parse!("REDIS_PORT"),
        postgres_user: env_get!("POSTGRES_USER"),
        postgres_password: env_get!("POSTGRES_PASSWORD"),
        postgres_host: env_get!("POSTGRES_HOST"),
        postgres_port: env_parse!("POSTGRES_PORT"),
        postgres_db: env_get!("POSTGRES_DB"),
    }
}
