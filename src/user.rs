use axum::{
    extract::{Path, State},
    routing::{put, delete, get, post},
    Router,
    response::{IntoResponse},
    http::StatusCode
};

use crate::state::SharedState;

pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub pswd_hash: String,
    pub pswd_salt: String,
    pub last_access: String,
}

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/users", get(handler_users))
        .route("/user", post(handler_add_user))
        .route("/user/:id", put(handler_modify_user))
        .route("/user/:id", delete(handler_delete_user))
}

async fn handler_users(State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: handler_users()");
    StatusCode::OK
}

async fn handler_add_user(State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: handler_add_user()");
    StatusCode::CREATED
}

async fn handler_modify_user(Path(id): Path<String>, State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: handler_modify_user({})", id);
    StatusCode::OK
}

async fn handler_delete_user(Path(id): Path<String>, State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: handler_delete_user({})", id);
    StatusCode::OK
}
