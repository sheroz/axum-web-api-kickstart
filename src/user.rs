use axum::{
    extract::{Path, State},
    response::Response,
    routing::{delete, get, post},
    Router,
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
        .route("/users", get(users))
        .route("/user", post(add_user))
        .route("/user/:id", delete(delete_user))
}

async fn users(State(_state): State<SharedState>) -> Response {
    todo!()
}

async fn add_user(State(_state): State<SharedState>) {
    todo!()
}

async fn delete_user(Path(_id): Path<String>, State(_state): State<SharedState>) {
    todo!()
}
