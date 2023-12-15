use crate::config;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub pgpool: Pool<Postgres>,
    pub redis: redis::Connection,
    pub config: config::Config,
}
