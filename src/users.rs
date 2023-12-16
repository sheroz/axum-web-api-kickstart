use axum::{extract::{Path, State}, routing::{put, delete, get, post}, Router, response::{IntoResponse}, http::StatusCode, Json};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, query_as};
use sqlx::types::Uuid;

use crate::state::SharedState;

#[allow(non_snake_case)]
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub pswd_hash: String,
    pub pswd_salt: String,
    pub last_access: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/", get(list_users_handler))
        .route("/", post(add_user_handler))
        .route("/:id", get(get_user_handler))
        .route("/:id", put(update_user_handler))
        .route("/:id", delete(delete_user_handler))
}

async fn list_users_handler(State(state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: handler_list_users()");

    if let Ok(users) = query_as::<_, User>("SELECT * FROM Users")
        .fetch_all(&state.pgpool).await {
        Json(users).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn add_user_handler(State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: handler_add_user()");
    StatusCode::CREATED
}

async fn get_user_handler(Path(id): Path<Uuid>, State(state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: handler_get_user({})", id);

    if let Ok(user) = sqlx::query_as::<_, User>("SELECT * FROM Users WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pgpool).await
    {
        Json(user).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn update_user_handler(Path(id): Path<String>, State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: handler_modify_user({})", id);
    StatusCode::OK
}

async fn delete_user_handler(Path(id): Path<String>, State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: handler_delete_user({})", id);
    StatusCode::OK
}
