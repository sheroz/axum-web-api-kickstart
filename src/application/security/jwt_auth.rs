use crate::{
    application::{
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

pub async fn logout(refresh_claims: JwtClaims, state: SharedState) -> Result<(), AuthError> {
    // checking the configuration if the usage of the list of revoked tokens is enabled
    if config::get().jwt_use_revoked_list {
        // decode and validate the refresh token
        if !validate_token_type(&refresh_claims, JwtTokenType::RefreshToken) {
            return Err(AuthError::InvalidToken);
        }
        revoke_refresh_token(&refresh_claims, &state).await
    } else {
        Err(AuthError::NotAcceptable)
    }
}

pub async fn refresh(
    refresh_claims: JwtClaims,
    state: SharedState,
) -> Result<JwtTokens, AuthError> {
    // decode and validate the refresh token
    if !validate_token_type(&refresh_claims, JwtTokenType::RefreshToken) {
        return Err(AuthError::InvalidToken);
    }

    // checking the configuration if the usage of the list of revoked tokens is enabled
    if config::get().jwt_use_revoked_list {
        revoke_refresh_token(&refresh_claims, &state).await?;
    }

    let user_id = refresh_claims.sub.parse().unwrap();
    if let Some(user) = user_repo::get_user(user_id, &state).await {
        let tokens = generate_tokens(user);
        return Ok(tokens);
    }
    Err(AuthError::InternalServerError)
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

pub fn validate_token_type(claims: &JwtClaims, expected_type: JwtTokenType) -> bool {
    if claims.typ == expected_type as u8 {
        return true;
    }
    tracing::error!(
        "Invalid token type. Expected {:?}, Found {:?}",
        expected_type,
        JwtTokenType::from(claims.typ),
    );
    false
}

async fn revoke_refresh_token(
    refresh_claims: &JwtClaims,
    state: &SharedState,
) -> Result<(), AuthError> {
    // check the validity of refresh token
    validate_revoked(refresh_claims, state).await?;
    if redis_service::revoke_refresh_token(refresh_claims, state).await {
        return Ok(());
    }
    Err(AuthError::InternalServerError)
}

pub fn generate_tokens(user: User) -> JwtTokens {
    let config = config::get();

    let time_now = chrono::Utc::now();
    let iat = time_now.timestamp() as usize;
    let sub = user.id.to_string();

    let access_token_id = Uuid::new_v4().to_string();
    let refresh_token_id = Uuid::new_v4().to_string();

    let access_claims = JwtClaims {
        sub: sub.clone(),
        jti: access_token_id.clone(),
        iat,
        exp: (time_now + chrono::Duration::seconds(config.jwt_expire_access_token_seconds))
            .timestamp() as usize,
        prf: refresh_token_id.clone(),
        typ: JwtTokenType::AccessToken as u8,
        roles: user.roles.clone(),
    };

    let access_token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &access_claims,
        &jsonwebtoken::EncodingKey::from_secret(config.jwt_secret.as_ref()),
    )
    .unwrap();

    let refresh_claims = JwtClaims {
        sub,
        jti: refresh_token_id,
        iat,
        exp: (time_now + chrono::Duration::seconds(config.jwt_expire_refresh_token_seconds))
            .timestamp() as usize,
        prf: access_token_id,
        typ: JwtTokenType::RefreshToken as u8,
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

pub async fn validate_revoked(claims: &JwtClaims, state: &SharedState) -> Result<(), AuthError> {
    match redis_service::is_revoked(claims, state).await {
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
