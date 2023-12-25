use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, RequestPartsExt, Router,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken as jwt;
use redis::{aio::Connection, AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::repository::user_repository::get_user_by_username,
    shared::{config, state::SharedState},
};

const REDIS_JWT_REVOKED: &str = "jwt.revoked";

#[derive(Debug, Serialize, Deserialize)]
struct LoginUser {
    username: String,
    password_hash: String,
}

/// [JWT Claims]
/// [RFC7519](https://datatracker.ietf.org/doc/html/rfc7519#section-4)
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject
    pub sub: String,
    /// JWT ID
    pub jti: String,
    /// Issued At
    pub iat: usize,
    /// Expiration Time
    pub exp: usize,
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    InternalServerError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
            AuthError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for JwtClaims
where
    SharedState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| {
                tracing::error!("invalid authorization header");
                AuthError::InvalidToken
            })?;

        // decode the user data
        let token_data = jwt::decode::<JwtClaims>(
            bearer.token(),
            &config::get().jwt_keys.decoding,
            &jwt::Validation::default(),
        )
        .map_err(|_| {
            tracing::error!("invalid token: {:#?}", bearer.token());
            AuthError::InvalidToken
        })?;

        // check Redis for revoked tokens
        let shared_state: SharedState = Arc::from_ref(state);
        let mut redis = shared_state.redis.lock().await;
        let redis_result: RedisResult<bool> = redis
            .sismember(REDIS_JWT_REVOKED, &token_data.claims.jti)
            .await;
        match redis_result {
            Ok(revoked) => {
                if revoked {
                    tracing::error!("access denied (revoked token): {:#?}", token_data.claims);
                    Err(AuthError::WrongCredentials)
                } else {
                    Ok(token_data.claims)
                }
            }
            Err(e) => {
                tracing::error!("{}", e);
                Err(AuthError::InternalServerError)
            }
        }
    }
}

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/logout", get(logout_handler))
        .route("/refresh", post(refresh_handler))
}

async fn refresh_handler() {
    tracing::debug!("entered: refresh_handler()");
}

async fn login_handler(
    State(state): State<SharedState>,
    Json(login): Json<LoginUser>,
) -> impl IntoResponse {
    tracing::debug!("entered: login_handler()");
    if let Some(user) = get_user_by_username(&login.username, &state).await {
        if user.password_hash == login.password_hash {
            let config = config::get();
            let time_now = chrono::Utc::now();

            let jwt_refresh_claims = JwtClaims {
                sub: user.id.to_string(),
                jti: Uuid::new_v4().to_string(),
                iat: time_now.timestamp() as usize,
                exp: (time_now + chrono::Duration::seconds(config.jwt_expire_refresh_token_seconds))
                    .timestamp() as usize,
            };

            let jwt_access_claims = JwtClaims {
                sub: user.id.to_string(),
                jti: Uuid::new_v4().to_string(),
                iat: time_now.timestamp() as usize,
                exp: (time_now + chrono::Duration::seconds(config.jwt_expire_access_token_seconds))
                    .timestamp() as usize,
            };

            let refresh_token = jwt::encode(
                &jwt::Header::default(),
                &jwt_refresh_claims,
                &jwt::EncodingKey::from_secret(config.jwt_secret.as_ref()),
            )
            .unwrap();

            let access_token = jwt::encode(
                &jwt::Header::default(),
                &jwt_access_claims,
                &jwt::EncodingKey::from_secret(config.jwt_secret.as_ref()),
            )
            .unwrap();

            tracing::info!("access granted: {:#?}", jwt_access_claims);

            let json = json!({
                "refresh_token": refresh_token,
                "access_token": access_token,
                "token_type": "Bearer"
            });
            tracing::trace!("granted token: {:#?}", json);

            return Json(json).into_response();
        }
    }

    tracing::error!("access denied: {:#?}", login);
    AuthError::WrongCredentials.into_response()
}

async fn logout_handler(claims: JwtClaims, State(state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: logout_handler()");
    tracing::trace!("logout claims: {:#?}", claims);

    // add token into revoked list in Redis
    // access tokens are tracked by JWT ID that handles the cases of reusing lost tokens and multi-device scenarios
    let mut redis = state.redis.lock().await;
    let redis_result: RedisResult<()> = redis.sadd(REDIS_JWT_REVOKED, claims.jti).await;
    if let Err(e) = redis_result {
        tracing::error!("{}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if tracing::enabled!(tracing::Level::TRACE) {
        log_revoked_jwt_tokens(&mut redis).await;
    }

    StatusCode::OK
}

async fn log_revoked_jwt_tokens(redis: &mut Connection) {
    let redis_result: RedisResult<Vec<String>> = redis.smembers(REDIS_JWT_REVOKED).await;
    match redis_result {
        Ok(revoked_tokens) => {
            tracing::trace!("redis -> revoked jwt tokens: {:#?}", revoked_tokens);
        }
        Err(e) => {
            tracing::error!("{}", e);
        }
    }
}
