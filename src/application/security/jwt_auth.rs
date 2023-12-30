use crate::{
    application::{
        app_const::*,
        redis_service,
        repository::user_repo,
        shared::{config, state::SharedState},
    },
    domain::models::user::User,
};
use uuid::Uuid;

use super::{auth_error::*, jwt_claims::*};

pub struct JwtTokens {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn logout(refresh_token: &str, state: &SharedState) -> Result<(), AuthError> {
    // checking the configuration if the usage of the list of revoked tokens is enabled
    if config::get().jwt_use_revoked_list {
        // decode and validate the refresh token
        let refresh_claims = decode_token(refresh_token, &jsonwebtoken::Validation::default())?;
        if !validate_token_type(&refresh_claims, JWT_JTI_PEFIX_REFRESH_TOKEN) {
            return Err(AuthError::InvalidToken);
        }
        revoke_refresh_token(&refresh_claims, state).await
    } else {
        Err(AuthError::NotAcceptable)
    }
}

pub async fn refresh(refresh_token: &str, state: &SharedState) -> Result<JwtTokens, AuthError> {
    // decode and validate the refresh token
    let refresh_claims = decode_token(refresh_token, &jsonwebtoken::Validation::default())?;
    if !validate_token_type(&refresh_claims, JWT_JTI_PEFIX_REFRESH_TOKEN) {
        return Err(AuthError::InvalidToken);
    }

    // checking the configuration if the usage of the list of revoked tokens is enabled
    if config::get().jwt_use_revoked_list {
        revoke_refresh_token(&refresh_claims, state).await?;
    }

    let user_id = decode_user_id(&refresh_claims)?;
    if let Some(user) = user_repo::get_user(user_id, state).await {
        let tokens = generate_tokens(user);
        return Ok(tokens);
    }
    Err(AuthError::InternalServerError)
}

pub fn decode_user_id(claims: &JwtClaims) -> Result<Uuid, AuthError> {
    if claims.jti.starts_with(JWT_JTI_PEFIX_ACCESS_TOKEN) {
        return Ok(claims.sub.parse().unwrap());
    }
    if claims.jti.starts_with(JWT_JTI_PEFIX_REFRESH_TOKEN) {
        let access_token = &claims.sub;
        let mut validation = jsonwebtoken::Validation::default();
        validation.validate_exp = false;
        // decode the access token
        let access_claims = decode_token(access_token, &validation)?;
        return Ok(access_claims.sub.parse().unwrap());
    }
    Err(AuthError::InvalidToken)
}

pub async fn cleanup_revoked_and_expired(
    _access_claims: &JwtClaims,
    state: &SharedState,
) -> Result<usize, AuthError> {
    // checking the configuration if the usage of the list of revoked tokens is enabled
    if !config::get().jwt_use_revoked_list {
        return Err(AuthError::NotAcceptable);
    }

    if let Some(deleted) = redis_service::cleanup_expired(state).await {
        return Ok(deleted);
    }
    Err(AuthError::InternalServerError)
}

pub fn validate_token_type(claims: &JwtClaims, expected_prefix: &str) -> bool {
    if claims.jti.starts_with(expected_prefix) {
        return true; 
    }
    tracing::error!(
        "Invalid token type. Expected {}, Found {}",
        JWT_JTI_PEFIX_REFRESH_TOKEN,
        &claims.jti[..2]
    );
    false
}

async fn revoke_refresh_token(
    refresh_claims: &JwtClaims,
    state: &SharedState,
) -> Result<(), AuthError> {

    // check the refresh token in revoked list
    validate_revoked(refresh_claims, state).await?;

    let mut claims_to_revoke = vec![refresh_claims];

    let access_claims;
    if let Ok(claims) = decode_token(&refresh_claims.sub, &jsonwebtoken::Validation::default()) {
        access_claims = claims;
        claims_to_revoke.push(&access_claims);
    }

    if redis_service::revoke_tokens(claims_to_revoke, state).await {
        return Ok(());
    }
    Err(AuthError::InternalServerError)
}

pub fn generate_tokens(user: User) -> JwtTokens {
    let config = config::get();
    let time_now = chrono::Utc::now();

    let access_claims = JwtClaims {
        sub: user.id.to_string(),
        jti: format!("{}:{}", JWT_JTI_PEFIX_ACCESS_TOKEN, Uuid::new_v4()),
        iat: time_now.timestamp() as usize,
        exp: (time_now + chrono::Duration::seconds(config.jwt_expire_access_token_seconds))
            .timestamp() as usize,
        roles: user.roles.clone(),
    };

    let access_token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &access_claims,
        &jsonwebtoken::EncodingKey::from_secret(config.jwt_secret.as_ref()),
    )
    .unwrap();

    let refresh_claims = JwtClaims {
        sub: access_token.clone(),
        jti: format!("{}:{}", JWT_JTI_PEFIX_REFRESH_TOKEN, Uuid::new_v4()),
        iat: time_now.timestamp() as usize,
        exp: (time_now + chrono::Duration::seconds(config.jwt_expire_refresh_token_seconds))
            .timestamp() as usize,
        roles: user.roles,
    };

    let refresh_token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &refresh_claims,
        &jsonwebtoken::EncodingKey::from_secret(config.jwt_secret.as_ref()),
    )
    .unwrap();

    tracing::info!(
        "JWT: generated claims\naccess {:#?}\nrefresh {:#?}",
        access_claims,
        refresh_claims
    );

    tracing::info!(
        "JWT: generated tokens\naccess {:#?}\nrefresh {:#?}",
        access_token,
        refresh_token
    );

    JwtTokens {
        access_token,
        refresh_token,
    }
}

fn decode_token(
    token: &str,
    validation: &jsonwebtoken::Validation,
) -> Result<JwtClaims, AuthError> {
    let jwt_keys = &config::get().jwt_keys;
    let token_data = jsonwebtoken::decode::<JwtClaims>(token, &jwt_keys.decoding, validation)
        .map_err(|_| {
            tracing::error!("Invalid token: {:#?}", token);
            AuthError::InvalidToken
        })?;

    Ok(token_data.claims)
}

pub async fn validate_revoked(claims: &JwtClaims, state: &SharedState) -> Result<(), AuthError> {
    let user_id = decode_user_id(claims)?;
    match redis_service::is_revoked(claims, user_id, state).await {
        Some(revoked) => {
            if revoked {
                return Err(AuthError::WrongCredentials);
            }
        }
        None => {
            return Err(AuthError::InternalServerError);
        }
    }
    Ok(())
}
