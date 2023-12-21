use crate::shared::config::Config;
use redis::aio::Connection;
use redis::aio::ConnectionManager;

pub async fn open(config: &Config) -> Connection {
    match redis::Client::open(config.redis_url()) {
        Ok(redis) => {
            match redis.get_async_connection().await {
                Ok(connection) => {
                    tracing::info!("Connected to redis");
                    connection
                }
                Err(e) => {
                    tracing::error!("Could not connect to redis: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            tracing::error!("Could not open redis: {}", e);
            std::process::exit(1);
        }
    }
}

pub async fn get_connection_manager(config: &Config) -> ConnectionManager {
    match redis::Client::open(config.redis_url()) {
        Ok(redis) => {
            match ConnectionManager::new(redis).await {
                Ok(connection) => {
                    tracing::info!("Connected to redis");
                    connection
                }
                Err(e) => {
                    tracing::error!("Could not connect to redis: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            tracing::error!("Could not open redis: {}", e);
            std::process::exit(1);
        }
    }
}
