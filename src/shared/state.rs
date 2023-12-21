use sqlx::{Pool, Postgres};
use std::sync::Arc;
use redis::aio::ConnectionManager;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub pgpool: Pool<Postgres>,
    pub redis: ConnectionManager,
    pub config: super::config::Config,
}
