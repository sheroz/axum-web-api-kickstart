use redis::{aio::MultiplexedConnection, AsyncCommands, RedisResult};
use std::collections::HashMap;
use tokio::sync::MutexGuard;

use super::{
    app_const::*,
    security::jwt_claims::{ClaimsMethods, RefreshClaims},
};
use crate::application::state::SharedState;

pub async fn revoke_global(state: &SharedState) -> bool {
    let timestamp_now = chrono::Utc::now().timestamp() as usize;
    tracing::debug!("setting a timestamp for global revoke: {}", timestamp_now);

    let mut redis = state.redis.lock().await;
    let redis_result: RedisResult<()> = redis
        .set(JWT_REDIS_REVOKE_GLOBAL_BEFORE_KEY, timestamp_now)
        .await;
    if let Err(e) = redis_result {
        tracing::error!("{}", e);
        return false;
    }
    true
}

pub async fn revoke_user_tokens(user_id: &str, state: &SharedState) -> bool {
    let timestamp_now = chrono::Utc::now().timestamp() as usize;
    tracing::debug!(
        "adding a timestamp for user revoke, user:{}, timestamp: {}",
        user_id,
        timestamp_now
    );

    let mut redis = state.redis.lock().await;
    let redis_result: RedisResult<()> = redis
        .hset(JWT_REDIS_REVOKE_USER_BEFORE_KEY, user_id, timestamp_now)
        .await;
    if let Err(e) = redis_result {
        tracing::error!("{}", e);
        return false;
    }
    true
}

async fn is_global_revoked<T: ClaimsMethods>(
    claims: &T,
    redis: &mut MutexGuard<'_, redis::aio::MultiplexedConnection>,
) -> Option<bool> {
    // check in global revoke
    let redis_result: RedisResult<Option<String>> =
        redis.get(JWT_REDIS_REVOKE_GLOBAL_BEFORE_KEY).await;
    match redis_result {
        Ok(opt_exp) => {
            if let Some(exp) = opt_exp {
                match exp.parse::<usize>() {
                    Ok(global_exp) => {
                        if global_exp >= claims.get_iat() {
                            return Some(true);
                        }
                    }
                    Err(e) => {
                        tracing::error!("{}", e);
                        return None;
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("{}", e);
            return None;
        }
    }
    Some(false)
}

async fn is_user_revoked<T: ClaimsMethods>(
    claims: &T,
    redis: &mut MutexGuard<'_, redis::aio::MultiplexedConnection>,
) -> Option<bool> {
    // check in user revoke
    let user_id = claims.get_sub();
    let redis_result: RedisResult<Option<String>> =
        redis.hget(JWT_REDIS_REVOKE_USER_BEFORE_KEY, user_id).await;
    match redis_result {
        Ok(opt_exp) => {
            if let Some(exp) = opt_exp {
                match exp.parse::<usize>() {
                    Ok(global_exp) => {
                        if global_exp >= claims.get_iat() {
                            return Some(true);
                        }
                    }
                    Err(e) => {
                        tracing::error!("{}", e);
                        return None;
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("{}", e);
            return None;
        }
    }
    Some(false)
}

async fn is_token_revoked<T: ClaimsMethods>(
    claims: &T,
    redis: &mut MutexGuard<'_, redis::aio::MultiplexedConnection>,
) -> Option<bool> {
    // check the token in revoked list
    let redis_result: RedisResult<bool> = redis
        .hexists(JWT_REDIS_REVOKED_TOKENS_KEY, claims.get_jti())
        .await;
    match redis_result {
        Ok(revoked) => Some(revoked),
        Err(e) => {
            tracing::error!("{}", e);
            None
        }
    }
}

pub async fn is_revoked<T: std::fmt::Debug + ClaimsMethods>(
    claims: &T,
    state: &SharedState,
) -> Option<bool> {
    let mut redis = state.redis.lock().await;
    match is_global_revoked(claims, &mut redis).await {
        Some(revoked) => {
            if revoked {
                tracing::error!("Access denied (globally revoked): {:#?}", claims);
                return Some(true);
            }
        }
        None => {
            return None;
        }
    }

    match is_user_revoked(claims, &mut redis).await {
        Some(revoked) => {
            if revoked {
                tracing::error!("Access denied (user revoked): {:#?}", claims);
                return Some(true);
            }
        }
        None => {
            return None;
        }
    }

    match is_token_revoked(claims, &mut redis).await {
        Some(revoked) => {
            if revoked {
                tracing::error!("Access denied (token revoked): {:#?}", claims);
                return Some(true);
            }
        }
        None => {
            return None;
        }
    }

    Some(false)
}

pub async fn revoke_refresh_token(claims: &RefreshClaims, state: &SharedState) -> bool {
    // adds the both refersh token and its paired access token into revoked list in Redis
    // tokens are tracked by JWT ID that handles the cases of reusing lost tokens and multi-device scenarios

    let mut redis = state.redis.lock().await;
    let list_to_revoke = vec![&claims.jti, &claims.prf];
    tracing::debug!("adding jwt tokens into revoked list: {:#?}", list_to_revoke);
    for claims_jti in list_to_revoke {
        let redis_result: RedisResult<()> = redis
            .hset(JWT_REDIS_REVOKED_TOKENS_KEY, claims_jti, claims.exp)
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

pub async fn cleanup_expired(state: &SharedState) -> Option<usize> {
    match delete_expired_tokens(state).await {
        Ok(deleted) => {
            tracing::debug!(
                "count of expired tokens deleted from the revoked list: {}",
                deleted
            );
            Some(deleted)
        }
        Err(e) => {
            tracing::error!("{}", e);
            None
        }
    }
}

async fn delete_expired_tokens(state: &SharedState) -> RedisResult<usize> {
    let timestamp_now = chrono::Utc::now().timestamp() as usize;

    let mut redis = state.redis.lock().await;
    let revoked_tokens: HashMap<String, String> =
        redis.hgetall(JWT_REDIS_REVOKED_TOKENS_KEY).await?;

    let mut deleted = 0;
    for (key, exp) in revoked_tokens {
        match exp.parse::<usize>() {
            Ok(timestamp_exp) => {
                if timestamp_now > timestamp_exp {
                    // Workaround for https://github.com/redis-rs/redis-rs/issues/1322
                    let _: () = redis.hdel(JWT_REDIS_REVOKED_TOKENS_KEY, key).await?;
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

pub async fn log_revoked_tokens_count(redis: &mut MultiplexedConnection) {
    let redis_result: RedisResult<usize> = redis.hlen(JWT_REDIS_REVOKED_TOKENS_KEY).await;
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

pub async fn log_revoked_tokens(redis: &mut MultiplexedConnection) {
    let redis_result: RedisResult<HashMap<String, String>> =
        redis.hgetall(JWT_REDIS_REVOKED_TOKENS_KEY).await;

    match redis_result {
        Ok(revoked_tokens) => {
            tracing::trace!("REDIS: list of revoked jwt tokens: {:#?}", revoked_tokens);
        }
        Err(e) => {
            tracing::error!("{}", e);
        }
    }
}
