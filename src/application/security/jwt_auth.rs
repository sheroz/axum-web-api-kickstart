use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    application::{app_const::*, redis_service},
    shared::{config, state::SharedState},
};

use super::{auth_error::*, jwt_claims::*};

pub async fn logout(claims: &JwtClaims, state: &SharedState) -> bool {
    if let Some(access_token_id) = parse_token_id(claims, JWT_JTI_PEFIX_ACCESS_TOKEN) {
        return redis_service::add_revoked(vec![access_token_id], state).await;
    }
    false
}

pub async fn refresh(refresh_token: &str, state: &SharedState) -> Result<Response, AuthError> {
    let jwt_keys = &config::get().jwt_keys;

    // decode the refresh token
    let refresh_token_data = jsonwebtoken::decode::<JwtClaims>(
        refresh_token,
        &jwt_keys.decoding,
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| {
        tracing::error!("invalid token: {:#?}", refresh_token);
        AuthError::InvalidToken
    })?;

    // validate the token type
    let refresh_token_id =
        match parse_token_id(&refresh_token_data.claims, JWT_JTI_PEFIX_REFRESH_TOKEN) {
            Some(id) => id,
            None => return Err(AuthError::InvalidToken),
        };

    let mut revoked_list = vec![refresh_token_id];

    // check the refresh token in revoked list
    match redis_service::exists_in_revoked(refresh_token_id, state).await {
        Some(revoked) => {
            if revoked {
                tracing::error!(
                    "access denied (revoked token): {:#?}",
                    refresh_token_data.claims
                );
                return Err(AuthError::WrongCredentials);
            }
        }
        None => {
            return Err(AuthError::InternalServerError);
        }
    }

    // decode the access token
    let access_token = &refresh_token_data.claims.sub;
    let mut validation = jsonwebtoken::Validation::default();
    validation.validate_exp = false;
    let access_token_data =
        jsonwebtoken::decode::<JwtClaims>(access_token, &jwt_keys.decoding, &validation).map_err(
            |_| {
                tracing::error!("invalid token: {:#?}", access_token);
                AuthError::InvalidToken
            },
        )?;

    // validate the token type
    let access_token_id =
        match parse_token_id(&access_token_data.claims, JWT_JTI_PEFIX_ACCESS_TOKEN) {
            Some(id) => id,
            None => return Err(AuthError::InvalidToken),
        };

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
    let user_id = access_token_data.claims.sub;
    build_response(user_id)
}

pub fn parse_token_id<'a>(claims: &'a JwtClaims, expected_type: &str) -> Option<&'a str> {
    // validate the token type
    let jti_field_len = claims.jti.len();
    if  jti_field_len == JWT_JTI_FIELD_SIZE {
        if claims.jti.starts_with(expected_type) {
            Some(claims.jti.get(2..).unwrap())
        } else {
            tracing::error!(
                "Could not parse token id. Invalid JWT ID type: found {}, expected {}",
                claims.jti,
                expected_type
            );
            None
        }
    }
    else {
        tracing::error!(
            "Could not parse token id. Invalid JWT ID size: found {}, expected {}",
            jti_field_len,
            JWT_JTI_FIELD_SIZE
        );
        None
    }    
}

pub fn build_response(user_id: String) -> Result<Response, AuthError> {
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

    let json = json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "token_type": "Bearer"
    });

    tracing::trace!("JWT: generated tokens {:#?}", json);
    Ok(Json(json).into_response())
}
