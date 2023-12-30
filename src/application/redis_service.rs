use std::collections::HashMap;

use super::{app_const::*, security::jwt_claims::JwtClaims};
use crate::application::shared::state::SharedState;
use redis::{aio::Connection, AsyncCommands, RedisResult};

pub async fn exists_in_revoked(claims: &JwtClaims, state: &SharedState) -> Option<bool> {
    // check the refresh token in revoked list
    let mut redis = state.redis.lock().await;
    let redis_result: RedisResult<bool> =
        redis.hexists(JWT_REDIS_REVOKED_LIST_KEY, &claims.jti).await;
    match redis_result {
        Ok(revoked) => Some(revoked),
        Err(e) => {
            tracing::error!("{}", e);
            None
        }
    }
}

pub async fn revoke_tokens(list_to_revoke: Vec<&JwtClaims>, state: &SharedState) -> bool {
    // add tokens into revoked list in Redis
    // tokens are tracked by JWT ID that handles the cases of reusing lost tokens and multi-device scenarios
    tracing::debug!("adding jwt tokens into revoked list: {:#?}", list_to_revoke);

    let mut redis = state.redis.lock().await;

    for claims in list_to_revoke {
        let redis_result: RedisResult<()> = redis
            .hset(JWT_REDIS_REVOKED_LIST_KEY, &claims.jti, claims.exp)
            .await;
        if let Err(e) = redis_result {
            tracing::error!("{}", e);
            return false;
        }
    }

    if tracing::enabled!(tracing::Level::TRACE) {
        log_revoked_tokens_count(&mut redis).await;
    }
    true
}

pub async fn cleanup_expired(state: &SharedState) -> bool {
    match delete_expired_tokens(state).await {
        Ok(deleted) => {
            tracing::debug!(
                "count of expired tokens deleted from the revoked list: {}",
                deleted
            );
            true
        }
        Err(e) => {
            tracing::error!("{}", e);
            false
        }
    }
}

async fn delete_expired_tokens(state: &SharedState) -> RedisResult<usize> {
    let timestamp_now = chrono::Utc::now().timestamp() as usize;

    let mut redis = state.redis.lock().await;
    let revoked_tokens: HashMap<String, String> = redis.hgetall(JWT_REDIS_REVOKED_LIST_KEY).await?;

    let mut deleted = 0;
    for (key, exp) in revoked_tokens {
        match exp.parse::<usize>() {
            Ok(timestamp_exp) => {
                if timestamp_now > timestamp_exp {
                    redis.hdel(JWT_REDIS_REVOKED_LIST_KEY, key).await?;
                    deleted += 1;
                }
            }
            Err(e) => {
                tracing::error!("{}", e);
            }
        }
    }

    if tracing::enabled!(tracing::Level::TRACE) {
        log_revoked_tokens_count(&mut redis).await;
    }

    Ok(deleted)
}

pub async fn log_revoked_tokens_count(redis: &mut Connection) {
    let redis_result: RedisResult<usize> = redis.hlen(JWT_REDIS_REVOKED_LIST_KEY).await;
    match redis_result {
        Ok(revoked_tokens_count) => {
            tracing::debug!(
                "REDIS: count of revoked jwt tokens: {}",
                revoked_tokens_count
            );
        }
        Err(e) => {
            tracing::error!("{}", e);
        }
    }
}

pub async fn log_revoked_tokens(redis: &mut Connection) {
    let redis_result: RedisResult<HashMap<String, String>> =
        redis.hgetall(JWT_REDIS_REVOKED_LIST_KEY).await;

    match redis_result {
        Ok(revoked_tokens) => {
            tracing::trace!("REDIS: list of revoked jwt tokens: {:#?}", revoked_tokens);
        }
        Err(e) => {
            tracing::error!("{}", e);
        }
    }
}
