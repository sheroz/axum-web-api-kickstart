use axum::{
    extract::{Path, State},
    routing::{put, delete, get, post},
    Router,
    response::IntoResponse,
    http::StatusCode, Json,
};
use sqlx::types::Uuid;

use crate::domain::model::user::User;
use crate::application::repository::user_repository::*;
use crate::shared::state::SharedState;

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
    match all_users(&state).await {
        Some(users) => Ok(Json(users)),
        None => Err(StatusCode::NOT_FOUND)
    }
}

async fn add_user_handler(State(state): State<SharedState>, Json(user): Json<User>) -> impl IntoResponse {
    tracing::debug!("entered: handler_add_user()");
    match add_user(user, &state).await {
        Some(user) => {
            (StatusCode::CREATED, Json(user)).into_response()
        }
        None => {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_user_handler(Path(id): Path<Uuid>, State(state): State<SharedState>) -> Result<Json<User>, impl IntoResponse> {
    tracing::debug!("entered: handler_get_user({})", id);
    match get_user(id, &state).await
    {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::NOT_FOUND)
    }
}

async fn update_user_handler(Path(id): Path<Uuid>, State(state): State<SharedState>, Json(user): Json<User>) -> Result<Json<User>, impl IntoResponse> {
    tracing::debug!("entered: update_user_handler({})", id);
    match update_user(id, user, &state).await {
        Some(user) => {
            Ok(Json(user))
        }
        None => {
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn delete_user_handler(Path(id): Path<Uuid>, State(state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: handler_delete_user({})", id);
    match delete_user(id, &state).await {
        Some(true) => StatusCode::OK,
        Some(false) => {
            tracing::warn!("User not found for deletion: {}", id);
            StatusCode::NOT_FOUND
        }
        None => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
