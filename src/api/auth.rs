use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use hyper::StatusCode;
use jsonwebtoken as jwt;
use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    application::repository::user_repository::get_user_by_username, shared::state::SharedState,
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
                &jwt::EncodingKey::from_secret(state.config.jwt_secret.as_ref()),
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

            let redis_result: RedisResult<Vec<String>> =
                redis.smembers("sessions".to_string()).await;
            match redis_result {
                Ok(sessions) => {
                    tracing::trace!("redis -> stored sessions: {:#?}", sessions);
                }
                Err(e) => {
                    tracing::error!("{}", e);
                    return StatusCode::FORBIDDEN.into_response();
                }
            }

            let json = json!({"access_token": access_token, "token_type": "Bearer"});
            return Json(json).into_response();
        }
    }
    StatusCode::FORBIDDEN.into_response()
}

async fn logout_handler(State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: logout_handler()");
    StatusCode::FORBIDDEN
}
