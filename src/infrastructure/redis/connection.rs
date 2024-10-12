use redis::aio::MultiplexedConnection;

use crate::application::config::Config;

pub async fn open(config: &Config) -> MultiplexedConnection {
    match redis::Client::open(config.redis_url()) {
        Ok(redis) => match redis.get_multiplexed_async_connection().await {
            Ok(connection) => {
                tracing::info!("Connected to redis");
                connection
            }
            Err(e) => {
                tracing::error!("Could not connect to redis: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            tracing::error!("Could not open redis: {}", e);
            std::process::exit(1);
        }
    }
}
