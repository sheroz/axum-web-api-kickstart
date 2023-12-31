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

use crate::application::{
    security::{self, auth_error::*},
    shared::{config, state::SharedState},
};

use super::jwt_auth;

/// [JWT Claims]
/// [RFC7519](https://datatracker.ietf.org/doc/html/rfc7519#section-4)
/// ToDo: implement role based validation: is_role(admin)
/// roles, groups: https://www.rfc-editor.org/rfc/rfc7643.html#section-4.1.2
/// https://www.rfc-editor.org/rfc/rfc9068.html#name-authorization-claims

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
    /// Reference to paired token
    pub prf: String,
    /// Token Type
    pub typ: u8,
    /// Roles
    pub roles: String,
}
impl JwtClaims {
    pub fn validate_role_admin(&self) -> Result<(), AuthError> {
        if !security::roles::is_role_admin(&self.roles) {
            return Err(AuthError::WrongCredentials);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum JwtTokenType {
    AccessToken,
    RefreshToken,
    UnknownToken,
}
impl From<u8> for JwtTokenType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::AccessToken,
            1 => Self::RefreshToken,
            2_u8..=u8::MAX => Self::UnknownToken,
        }
    }
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
                tracing::error!("Invalid authorization header");
                AuthError::WrongCredentials
            })?;

        // decode the user data
        let token_data = jsonwebtoken::decode::<JwtClaims>(
            bearer.token(),
            &config::get().jwt_keys.decoding,
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|_| {
            tracing::error!("Invalid token: {:#?}", bearer.token());
            AuthError::WrongCredentials
        })?;

        // check for revoked tokens if enabled by configuration
        if config::get().jwt_use_revoked_list {
            let shared_state: SharedState = Arc::from_ref(state);
            jwt_auth::validate_revoked(&token_data.claims, &shared_state).await?
        }
        Ok(token_data.claims)
    }
}
