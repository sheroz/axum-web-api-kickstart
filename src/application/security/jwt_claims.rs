use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    application::{app_const::*, redis_service, security::auth_error::*},
    shared::{config, state::SharedState},
};

use super::jwt_auth;

/// [JWT Claims]
/// [RFC7519](https://datatracker.ietf.org/doc/html/rfc7519#section-4)
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject
    pub sub: String,
    /// JWT ID
    pub jti: String,
    /// Issued At
    pub iat: usize,
    /// Expiration Time
    pub exp: usize,
}

#[async_trait]
impl<S> FromRequestParts<S> for JwtClaims
where
    SharedState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| {
                tracing::error!("invalid authorization header");
                AuthError::InvalidToken
            })?;

        // decode the user data
        let token_data = jsonwebtoken::decode::<JwtClaims>(
            bearer.token(),
            &config::get().jwt_keys.decoding,
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|_| {
            tracing::error!("invalid token: {:#?}", bearer.token());
            AuthError::InvalidToken
        })?;

        // check for revoked tokens
        let shared_state: SharedState = Arc::from_ref(state);
        let access_token_id = jwt_auth::parse_token_id(&token_data.claims, JWT_JTI_PEFIX_ACCESS_TOKEN)?;

        match redis_service::exists_in_revoked(access_token_id, &shared_state).await {
            Some(revoked) => {
                if revoked {
                    tracing::error!("access denied (revoked token): {:#?}", token_data.claims);
                    Err(AuthError::WrongCredentials)
                } else {
                    Ok(token_data.claims)
                }
            }
            None => Err(AuthError::InternalServerError),
        }
    }
}
