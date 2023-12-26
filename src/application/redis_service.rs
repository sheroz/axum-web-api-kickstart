use redis::{aio::Connection, AsyncCommands, RedisResult};

use crate::shared::state::SharedState;

use super::app_const::*;

pub async fn exists_in_revoked(token_id: &str, state: &SharedState) -> Option<bool> {
    // check the refresh token in revoked list
    let mut redis = state.redis.lock().await;
    let redis_result: RedisResult<bool> =
        redis.sismember(JWT_REDIS_REVOKED_LIST_KEY, token_id).await;
    match redis_result {
        Ok(revoked) => Some(revoked),
        Err(e) => {
            tracing::error!("{}", e);
            None
        }
    }
}

pub async fn add_revoked(revoked_ids: Vec<&str>, state: &SharedState) -> bool {
    // add tokens into revoked list in Redis
    // tokens are tracked by JWT ID that handles the cases of reusing lost tokens and multi-device scenarios
    let mut redis = state.redis.lock().await;

    for token_id in revoked_ids {
        let redis_result: RedisResult<()> = redis.sadd(JWT_REDIS_REVOKED_LIST_KEY, token_id).await;
        if let Err(e) = redis_result {
            tracing::error!("{}", e);
            return false;
        }
    }

    if tracing::enabled!(tracing::Level::TRACE) {
        log_revoked_tokens(&mut redis).await;
    }
    true
}

pub async fn log_revoked_tokens(redis: &mut Connection) {
    let redis_result: RedisResult<Vec<String>> = redis.smembers(JWT_REDIS_REVOKED_LIST_KEY).await;
    match redis_result {
        Ok(revoked_tokens) => {
            tracing::trace!("redis -> revoked jwt tokens: {:#?}", revoked_tokens);
        }
        Err(e) => {
            tracing::error!("{}", e);
        }
    }
}
