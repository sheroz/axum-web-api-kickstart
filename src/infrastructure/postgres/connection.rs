use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::application::shared::config::Config;

pub async fn pgpool(config: &Config) -> Pool<Postgres> {
    match PgPoolOptions::new()
        .max_connections(config.postgres_connection_pool)
        .connect(&config.postgres_url())
        .await
    {
        Ok(pool) => {
            tracing::info!("Connected to postgres");
            pool
        }
        Err(e) => {
            tracing::error!("Could not connect to postgres: {}", e);
            std::process::exit(1);
        }
    }
}
