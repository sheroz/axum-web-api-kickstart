use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub pgpool: Pool<Postgres>,
    pub redis: Mutex<redis::aio::Connection>,
}
