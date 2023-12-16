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
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
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

async fn list_users_handler(State(state): State<SharedState>) -> Result<Json<Vec<User>>, impl IntoResponse> {
    tracing::debug!("entered: handler_list_users()");

    if let Ok(users) = query_as::<_, User>("SELECT * FROM Users")
        .fetch_all(&state.pgpool).await {
        Ok(Json(users))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn add_user_handler(State(state): State<SharedState>, Json(user): Json<User>) -> impl IntoResponse {
    tracing::debug!("entered: handler_add_user()");
    tracing::trace!("user: {:#?}", user);
    let result = sqlx::query("INSERT INTO Users (id, username, email, pswd_hash, pswd_salt) values ($1,$2,$3,$4,$5)")
        .bind(user.id)
        .bind(user.username)
        .bind(user.email)
        .bind(user.pswd_hash)
        .bind(user.pswd_salt)
        .execute(&state.pgpool)
        .await;
    match result {
        Ok(row) => {
            if row.rows_affected() == 1 {
                StatusCode::CREATED
            } else {
                StatusCode::OK
            }
        }
        Err(e) => {
            tracing::error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn get_user_handler(Path(id): Path<Uuid>, State(state): State<SharedState>) -> Result<Json<User>, impl IntoResponse> {
    tracing::debug!("entered: handler_get_user({})", id);

    if let Ok(user) = sqlx::query_as::<_, User>("SELECT * FROM Users WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pgpool).await
    {
        Ok(Json(user))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn update_user_handler(Path(id): Path<Uuid>, State(_state): State<SharedState>, Json(_user): Json<User>) -> impl IntoResponse {
    tracing::debug!("entered: update_user_handler({})", id);
    StatusCode::OK
}

async fn delete_user_handler(Path(id): Path<Uuid>, State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: handler_delete_user({})", id);
    StatusCode::OK
}
