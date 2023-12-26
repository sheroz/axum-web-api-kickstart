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

pub async fn logout(
    access_claims: &JwtClaims,
    refresh_token: &str,
    state: &SharedState,
) -> Result<(), AuthError> {
    let access_token_id = parse_token_id(access_claims, JWT_JTI_PEFIX_ACCESS_TOKEN)?;
    let refresh_claims = decode_token(refresh_token, &jsonwebtoken::Validation::default())?;
    let refresh_token_id = parse_token_id(&refresh_claims, JWT_JTI_PEFIX_REFRESH_TOKEN)?;
    let revoke_ids = vec![access_token_id, refresh_token_id];
    if redis_service::add_revoked(revoke_ids, state).await {
        return Ok(());
    }
    Err(AuthError::InternalServerError)
}

fn decode_token(
    token: &str,
    validation: &jsonwebtoken::Validation,
) -> Result<JwtClaims, AuthError> {
    let jwt_keys = &config::get().jwt_keys;
    let token_data = jsonwebtoken::decode::<JwtClaims>(token, &jwt_keys.decoding, validation)
        .map_err(|_| {
            tracing::error!("invalid token: {:#?}", token);
            AuthError::InvalidToken
        })?;

    Ok(token_data.claims)
}

pub fn parse_token_id<'a>(
    claims: &'a JwtClaims,
    expected_type: &str,
) -> Result<&'a str, AuthError> {
    let jti_field_len = claims.jti.len();
    // check the JWT ID size
    if jti_field_len == JWT_JTI_FIELD_SIZE {
        // check the JWT ID type
        if claims.jti.starts_with(expected_type) {
            return Ok(claims.jti.get(2..).unwrap());
        } else {
            tracing::error!(
                "Could not parse token id. Invalid JWT ID type: found {}, expected {}",
                claims.jti,
                expected_type
            );
        }
    } else {
        tracing::error!(
            "Could not parse token id. Invalid JWT ID size: found {}, expected {}",
            jti_field_len,
            JWT_JTI_FIELD_SIZE
        );
    }
    Err(AuthError::InvalidToken)
}

pub async fn refresh(refresh_token: &str, state: &SharedState) -> Result<JwtTokens, AuthError> {
    let jwt_keys = &config::get().jwt_keys;

    // decode the refresh token
    let refresh_claims = decode_token(refresh_token, &jsonwebtoken::Validation::default())?;

    // validate the token type
    let refresh_token_id = parse_token_id(&refresh_claims, JWT_JTI_PEFIX_REFRESH_TOKEN)?;

    let mut revoked_list = vec![refresh_token_id];

    // check the refresh token in revoked list
    match redis_service::exists_in_revoked(refresh_token_id, state).await {
        Some(revoked) => {
            if revoked {
                tracing::error!("access denied (revoked token): {:#?}", refresh_claims);
                return Err(AuthError::WrongCredentials);
            }
        }
        None => {
            return Err(AuthError::InternalServerError);
        }
    }

    // decode the access token
    let access_token = &refresh_claims.sub;
    let mut validation = jsonwebtoken::Validation::default();
    validation.validate_exp = false;
    let access_token_claims = decode_token(access_token, &validation)?;

    // validate the token type
    let access_token_id = parse_token_id(&access_token_claims, JWT_JTI_PEFIX_ACCESS_TOKEN)?;

    // validate the expiry time of access token
    validation.validate_exp = true;
    if jsonwebtoken::decode::<JwtClaims>(access_token, &jwt_keys.decoding, &validation).is_ok() {
        // access token not expired yet, needs revoked
        revoked_list.push(access_token_id);
    }

    if !redis_service::add_revoked(revoked_list, state).await {
        return Err(AuthError::InternalServerError);
    }

    // using refresh token rotation technique
    let user_id = access_token_claims.sub;
    let tokens = generate_tokens(user_id);
    Ok(tokens)
}

pub fn generate_tokens(user_id: String) -> JwtTokens {
    let config = config::get();
    let time_now = chrono::Utc::now();

    let access_claims = JwtClaims {
        sub: user_id,
        jti: format!("{}{}", JWT_JTI_PEFIX_ACCESS_TOKEN, Uuid::new_v4()),
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
        jti: format!("{}{}", JWT_JTI_PEFIX_REFRESH_TOKEN, Uuid::new_v4()),
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
