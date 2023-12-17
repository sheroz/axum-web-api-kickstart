use axum::{extract::{Path, State}, routing::{put, delete, get, post}, Router, response::{IntoResponse}, http::StatusCode, Json};
use chrono::{NaiveDateTime, Utc};
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
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/", get(list_users_handler))
        .route("/", post(add_user_handler))
        .route("/:id", get(get_user_handler))
        .route("/:id", put(update_user_handler))
        .route("/:id", delete(delete_user_handler))
}

async fn list_users_handler(State(state): State<SharedState>) -> Result<Json<Vec<User>>, impl IntoResponse> {
    tracing::debug!("entered: handler_list_users()");

    if let Ok(users) = query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&state.pgpool).await {
        Ok(Json(users))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn add_user_handler(State(state): State<SharedState>, Json(user): Json<User>) -> impl IntoResponse {
    tracing::debug!("entered: handler_add_user()");
    let time_now = Utc::now().naive_utc();
    tracing::trace!("user: {:#?}", user);
    let query_result = sqlx::query_as::<_, User>(
        r#"INSERT INTO users (id,
         username,
         email,
         pswd_hash,
         pswd_salt,
         created_at,
         updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7)
         RETURNING users.*"#)
        .bind(user.id)
        .bind(user.username)
        .bind(user.email)
        .bind(user.pswd_hash)
        .bind(user.pswd_salt)
        .bind(time_now)
        .bind(time_now)
        .fetch_one(&state.pgpool)
        .await;

    match query_result {
        Ok(user) => {
            (StatusCode::CREATED, Json(user)).into_response()
        }
        Err(e) => {
            tracing::error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_user_handler(Path(id): Path<Uuid>, State(state): State<SharedState>) -> Result<Json<User>, impl IntoResponse> {
    tracing::debug!("entered: handler_get_user({})", id);

    if let Ok(user) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pgpool).await
    {
        Ok(Json(user))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn update_user_handler(Path(id): Path<Uuid>, State(state): State<SharedState>, Json(user): Json<User>) -> Result<Json<User>, impl IntoResponse> {
    tracing::debug!("entered: update_user_handler({})", id);
    let time_now = Utc::now().naive_utc();
    tracing::trace!("user: {:#?}", user);
    let query_result = sqlx::query_as::<_, User>(
        r#"UPDATE users
         SET username = $1,
         email = $2,
         pswd_hash = $3,
         pswd_salt = $4,
         updated_at = $5
         WHERE id = $6
         RETURNING users.*"#)
        .bind(user.username)
        .bind(user.email)
        .bind(user.pswd_hash)
        .bind(user.pswd_salt)
        .bind(time_now)
        .bind(user.id)
        .fetch_one(&state.pgpool)
        .await;

    match query_result {
        Ok(user) => {
            Ok(Json(user))
        }
        Err(e) => {
            tracing::error!("{}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn delete_user_handler(Path(id): Path<Uuid>, State(state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: handler_delete_user({})", id);
    let query_result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.pgpool)
        .await;

    match query_result {
        Ok(row) => {
            if row.rows_affected() == 1 {
                StatusCode::OK
            } else {
                tracing::warn!("User not found for deletion: {}", id);
                StatusCode::NOT_FOUND
            }
        }
        Err(e) => {
            tracing::error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
