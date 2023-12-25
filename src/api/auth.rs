use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    application::{
        repository::user_repo,
        security::{auth_error::AuthError, jwt_auth, jwt_claims::JwtClaims},
    },
    shared::state::SharedState,
};

#[derive(Debug, Serialize, Deserialize)]
struct LoginUser {
    username: String,
    password_hash: String,
}

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/logout", get(logout_handler))
        .route("/refresh", post(refresh_handler))
}

async fn refresh_handler(
    State(state): State<SharedState>,
    refresh_token: String,
) -> Result<Response, AuthError> {
    tracing::debug!("entered: refresh_handler()");
    jwt_auth::refresh(&refresh_token, &state).await
}

async fn login_handler(
    State(state): State<SharedState>,
    Json(login): Json<LoginUser>,
) -> Result<Response, AuthError> {
    tracing::debug!("entered: login_handler()");
    if let Some(user) = user_repo::get_user_by_username(&login.username, &state).await {
        if user.password_hash == login.password_hash {
            tracing::trace!("access granted: {}", user.id);
            return jwt_auth::build_response(user.id.to_string());
        }
    }

    tracing::error!("access denied: {:#?}", login);
    Err(AuthError::WrongCredentials)
}

async fn logout_handler(claims: JwtClaims, State(state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: logout_handler()");
    tracing::trace!("logout claims: {:#?}", claims);
    
    // !!! needs to revoke the refresh token !!!
    if jwt_auth::logout(&claims, &state).await {
        return StatusCode::OK;
    }
    StatusCode::INTERNAL_SERVER_ERROR
}
