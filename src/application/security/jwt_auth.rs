use crate::{
    application::{app_const::*, redis_service},
    shared::{config, state::SharedState},
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
        let refresh_claims = decode_token(refresh_token, &jsonwebtoken::Validation::default())?;
        revoke_refresh_token(&refresh_claims, state).await
    } else {
        Err(AuthError::NotAcceptable)
    }
}

pub async fn refresh(refresh_token: &str, state: &SharedState) -> Result<JwtTokens, AuthError> {
    // decode the refresh token
    let refresh_claims = decode_token(refresh_token, &jsonwebtoken::Validation::default())?;

    // checking the configuration if the usage of the list of revoked tokens is enabled
    if config::get().jwt_use_revoked_list {
        revoke_refresh_token(&refresh_claims, state).await?;
    }

    // decode the access token
    let access_token = &refresh_claims.sub;
    let mut validation = jsonwebtoken::Validation::default();
    validation.validate_exp = false;
    let access_claims = decode_token(access_token, &validation)?;

    // using refresh token rotation technique
    let user_id = access_claims.sub;
    let tokens = generate_tokens(user_id);
    Ok(tokens)
}

pub async fn cleanup_revoked_and_expired(
    _access_claims: &JwtClaims,
    state: &SharedState,
) -> Result<(), AuthError> {
    // checking the configuration if the usage of the list of revoked tokens is enabled
    if !config::get().jwt_use_revoked_list {
        return Err(AuthError::NotAcceptable);
    }

    if !redis_service::cleanup_expired(state).await {
        return Err(AuthError::InternalServerError);
    }
    Ok(())
}

async fn revoke_refresh_token(
    refresh_claims: &JwtClaims,
    state: &SharedState,
) -> Result<(), AuthError> {
    if !refresh_claims.jti.starts_with(JWT_JTI_PEFIX_REFRESH_TOKEN) {
        tracing::error!(
            "Invalid token type. Expected {}, Found {}",
            JWT_JTI_PEFIX_REFRESH_TOKEN,
            &refresh_claims.jti[..2]
        );
        return Err(AuthError::InvalidToken);
    }

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

pub fn generate_tokens(user_id: String) -> JwtTokens {
    let config = config::get();
    let time_now = chrono::Utc::now();

    let access_claims = JwtClaims {
        sub: user_id,
        jti: format!("{}:{}", JWT_JTI_PEFIX_ACCESS_TOKEN, Uuid::new_v4()),
        iat: time_now.timestamp() as usize,
        exp: (time_now + chrono::Duration::seconds(config.jwt_expire_access_token_seconds))
            .timestamp() as usize,
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

async fn validate_revoked(claims: &JwtClaims, state: &SharedState) -> Result<(), AuthError> {
    match redis_service::exists_in_revoked(claims, state).await {
        Some(revoked) => {
            if revoked {
                tracing::error!("Access denied (revoked token): {:#?}", claims);
                return Err(AuthError::WrongCredentials);
            }
        }
        None => {
            return Err(AuthError::InternalServerError);
        }
    }
    Ok(())
}
