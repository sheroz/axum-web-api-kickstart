use axum::{
    async_trait,
    extract::{FromRequestParts, State},
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

use crate::{
    application::repository::user_repository::get_user_by_username,
    shared::{config, state::SharedState},
};

#[derive(Debug, Serialize, Deserialize)]
struct LoginUser {
    username: String,
    password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
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
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        // Decode the user data
        let token_data = jwt::decode::<JwtClaims>(
            bearer.token(),
            &config::get().jwt_keys.decoding,
            &jwt::Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/logout", get(logout_handler))
}

async fn login_handler(
    State(state): State<SharedState>,
    Json(login): Json<LoginUser>,
) -> impl IntoResponse {
    tracing::debug!("entered: login_handler()");
    tracing::trace!("login: {:#?}", login);
    if let Some(user) = get_user_by_username(&login.username, &state).await {
        if user.password_hash == login.password_hash {
            let time_now = chrono::Utc::now();
            let jwt_claims = JwtClaims {
                sub: user.id.to_string(),
                iat: time_now.timestamp() as usize,
                exp: (time_now + chrono::Duration::minutes(60)).timestamp() as usize,
            };

            let access_token = jwt::encode(
                &jwt::Header::default(),
                &jwt_claims,
                &jwt::EncodingKey::from_secret(config::get().jwt_secret.as_ref()),
            )
            .unwrap();

            let mut redis = state.redis.lock().await;
            let redis_result: RedisResult<()> = redis
                .sadd("sessions".to_string(), user.id.to_string())
                .await;
            if let Err(e) = redis_result {
                tracing::error!("{}", e);
                return StatusCode::FORBIDDEN.into_response();
            }

            if tracing::enabled!(tracing::Level::TRACE) {
                log_sessions(&mut redis).await;
            }

            let json = json!({"access_token": access_token, "token_type": "Bearer"});
            return Json(json).into_response();
        }
    }
    StatusCode::FORBIDDEN.into_response()
}

async fn logout_handler(claims: JwtClaims, State(state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: logout_handler()");
    tracing::trace!("logout claims: {:#?}", claims);

    // removing session data from Redis
    let mut redis = state.redis.lock().await;
    let redis_result: RedisResult<()> = redis.srem("sessions".to_string(), claims.sub).await;
    if let Err(e) = redis_result {
        tracing::error!("{}", e);
        return StatusCode::FORBIDDEN;
    }

    if tracing::enabled!(tracing::Level::TRACE) {
        log_sessions(&mut redis).await;
    }

    StatusCode::OK
}

async fn log_sessions(redis: &mut Connection) {
    let redis_result: RedisResult<Vec<String>> = redis.smembers("sessions".to_string()).await;
    match redis_result {
        Ok(sessions) => {
            tracing::trace!("redis -> stored sessions: {:#?}", sessions);
        }
        Err(e) => {
            tracing::error!("{}", e);
        }
    }
}
