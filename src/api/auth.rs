use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    application::{
        repository::user_repo,
        security::{
            auth_error::AuthError,
            jwt_auth::{self, JwtTokens}, jwt_claims::JwtClaims,
        },
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
        .route("/logout", post(logout_handler))
        .route("/refresh", post(refresh_handler))
        .route("/clean-up", post(cleanup_handler))
}

async fn refresh_handler(
    State(state): State<SharedState>,
    refresh_token: String,
) -> Result<Response, AuthError> {
    tracing::debug!("entered: refresh_handler()");
    let tokens = jwt_auth::refresh(&refresh_token, &state).await?;
    let response = tokens_to_response(tokens);
    Ok(response)
}

async fn login_handler(
    State(state): State<SharedState>,
    Json(login): Json<LoginUser>,
) -> Result<Response, AuthError> {
    tracing::debug!("entered: login_handler()");
    if let Some(user) = user_repo::get_user_by_username(&login.username, &state).await {
        if user.password_hash == login.password_hash {
            tracing::trace!("access granted: {}", user.id);
            let tokens = jwt_auth::generate_tokens(user.id.to_string());
            let response = tokens_to_response(tokens);
            return Ok(response);
        }
    }

    tracing::error!("access denied: {:#?}", login);
    Err(AuthError::WrongCredentials)
}

async fn logout_handler(
    State(state): State<SharedState>,
    refresh_token: String,
) -> impl IntoResponse {
    tracing::debug!("entered: logout_handler()");
    tracing::trace!("refresh_token: {}", refresh_token);
    jwt_auth::logout(&refresh_token, &state).await
}

async fn cleanup_handler(
    State(state): State<SharedState>,
    access_claims: JwtClaims
) -> impl IntoResponse {
    tracing::debug!("entered: cleanup_handler()");
    tracing::trace!("authentication details: {:#?}", access_claims);
    jwt_auth::cleanup_revoked_and_expired(&access_claims, &state).await
}

fn tokens_to_response(jwt_tokens: JwtTokens) -> Response {
    let json = json!({
        "access_token": jwt_tokens.access_token,
        "refresh_token": jwt_tokens.refresh_token,
        "token_type": "Bearer"
    });

    tracing::trace!("JWT: generated response {:#?}", json);
    Json(json).into_response()
}
