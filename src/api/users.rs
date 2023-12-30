use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use sqlx::types::Uuid;

use crate::{
    application::{
        repository::user_repo, security::jwt_claims::JwtClaims, shared::state::SharedState,
    },
    domain::models::user::User,
};

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/", get(list_users_handler))
        .route("/", post(add_user_handler))
        .route("/:id", get(get_user_handler))
        .route("/:id", put(update_user_handler))
        .route("/:id", delete(delete_user_handler))
}

async fn list_users_handler(
    access_claims: JwtClaims,
    State(state): State<SharedState>,
) -> Result<Json<Vec<User>>, impl IntoResponse> {
    // ToDo: check access control
    tracing::trace!("authentication details: {:#?}", access_claims);
    match user_repo::all_users(&state).await {
        Some(users) => Ok(Json(users)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn add_user_handler(
    access_claims: JwtClaims,
    State(state): State<SharedState>,
    Json(user): Json<User>,
) -> impl IntoResponse {
    tracing::trace!("authentication details: {:#?}", access_claims);
    match user_repo::add_user(user, &state).await {
        Some(user) => (StatusCode::CREATED, Json(user)).into_response(),
        None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_user_handler(
    access_claims: JwtClaims,
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
) -> Result<Json<User>, impl IntoResponse> {
    // ToDo: check access control
    tracing::trace!("id: {}", id);
    tracing::trace!("authentication details: {:#?}", access_claims);
    match user_repo::get_user(id, &state).await {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn update_user_handler(
    access_claims: JwtClaims,
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
    Json(user): Json<User>,
) -> Result<Json<User>, impl IntoResponse> {
    // ToDo: check access control
    tracing::trace!("id: {}", id);
    tracing::trace!("authentication details: {:#?}", access_claims);
    match user_repo::update_user(id, user, &state).await {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn delete_user_handler(
    access_claims: JwtClaims,
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    // ToDo: check access control
    tracing::trace!("id: {}", id);
    tracing::trace!("authentication details: {:#?}", access_claims);
    match user_repo::delete_user(id, &state).await {
        Some(true) => StatusCode::OK,
        Some(false) => {
            tracing::warn!("User not found for deletion: {}", id);
            StatusCode::NOT_FOUND
        }
        None => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
