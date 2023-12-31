use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::application::{
    redis_service,
    repository::user_repo,
    security::{
        auth_error::AuthError,
        jwt_auth::{self, JwtTokens},
        jwt_claims::{AccessClaims, RefreshClaims, ClaimsMethods}
    },
    shared::state::SharedState,
};

#[derive(Debug, Serialize, Deserialize)]
struct LoginUser {
    username: String,
    password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RevokeUser {
    user_id: Uuid,
}

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/refresh", post(refresh_handler))
        .route("/revoke_all", post(revoke_all_handler))
        .route("/revoke_user", post(revoke_user_handler))
        .route("/clean-up", post(cleanup_handler))
}

async fn login_handler(
    State(state): State<SharedState>,
    Json(login): Json<LoginUser>,
) -> Result<Response, AuthError> {
    if let Some(user) = user_repo::get_user_by_username(&login.username, &state).await {
        if user.active && user.password_hash == login.password_hash {
            tracing::trace!("access granted, user: {}", user.id);
            let tokens = jwt_auth::generate_tokens(user);
            let response = tokens_to_response(tokens);
            return Ok(response);
        }
    }

    tracing::error!("access denied: {:#?}", login);
    Err(AuthError::WrongCredentials)
}

async fn logout_handler(
    State(state): State<SharedState>,
    refresh_claims: RefreshClaims,
) -> impl IntoResponse {
    tracing::trace!("refresh_claims: {:?}", refresh_claims);
    jwt_auth::logout(refresh_claims, state).await
}

async fn refresh_handler(
    State(state): State<SharedState>,
    refresh_claims: RefreshClaims,
) -> Result<Response, AuthError> {
    let new_tokens = jwt_auth::refresh(refresh_claims, state).await?;
    let response = tokens_to_response(new_tokens);
    Ok(response)
}

// revoke all issued tokens until now
async fn revoke_all_handler(
    State(state): State<SharedState>,
    access_claims: AccessClaims,
) -> impl IntoResponse {
    access_claims.validate_role_admin()?;
    if !redis_service::revoke_global(&state).await {
        return Err(AuthError::InternalServerError);
    }
    Ok(())
}

// revoke tokens issued to user until now
async fn revoke_user_handler(
    State(state): State<SharedState>,
    access_claims: AccessClaims,
    Json(revoke_user): Json<RevokeUser>,
) -> impl IntoResponse {
    if access_claims.sub != revoke_user.user_id.to_string() {
        // only admin can revoke tokens of other users
        access_claims.validate_role_admin()?;
    }
    tracing::trace!("revoke_user: {:?}", revoke_user);
    if !redis_service::revoke_user_tokens(&revoke_user.user_id.to_string(), &state).await {
        return Err(AuthError::InternalServerError);
    }
    Ok(())
}

async fn cleanup_handler(
    State(state): State<SharedState>,
    access_claims: AccessClaims,
) -> Result<Response, AuthError> {
    access_claims.validate_role_admin()?;
    tracing::trace!("authentication details: {:#?}", access_claims);
    let deleted = jwt_auth::cleanup_revoked_and_expired(&access_claims, &state).await?;
    let json = json!({
        "deleted_tokens": deleted,
    });
    Ok(Json(json).into_response())
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
