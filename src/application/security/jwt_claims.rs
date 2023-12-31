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
pub struct AccessClaims {
    /// Subject
    pub sub: String,
    /// JWT ID
    pub jti: String,
    /// Issued At
    pub iat: usize,
    /// Expiration Time
    pub exp: usize,
    /// Token Type
    pub typ: u8,
    /// Roles
    pub roles: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    /// Subject
    pub sub: String,
    /// JWT ID
    pub jti: String,
    /// Issued At
    pub iat: usize,
    /// Expiration Time
    pub exp: usize,
    /// Reference to paired access token
    pub prf: String,
    /// Expiration time of paired access token
    pub pex: usize,
    /// Token Type
    pub typ: u8,
    /// Roles
    pub roles: String,
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

pub trait ClaimsMethods {
    fn validate_role_admin(&self) -> Result<(), AuthError>;
    fn get_sub(&self) -> &str;
    fn get_exp(&self) -> usize;
    fn get_iat(&self) -> usize;
    fn get_jti(&self) -> &str;
}

impl ClaimsMethods for AccessClaims {
    fn validate_role_admin(&self) -> Result<(), AuthError> {
        is_role_admin(&self.roles)
    }
    fn get_sub(&self) -> &str {
        &self.sub
    }

    fn get_iat(&self) -> usize {
        self.iat
    }

    fn get_exp(&self) -> usize {
        self.exp
    }

    fn get_jti(&self) -> &str {
        &self.jti
    }
}
impl ClaimsMethods for RefreshClaims {
    fn validate_role_admin(&self) -> Result<(), AuthError> {
        is_role_admin(&self.roles)
    }
    fn get_sub(&self) -> &str {
        &self.sub
    }

    fn get_iat(&self) -> usize {
        self.iat
    }

    fn get_exp(&self) -> usize {
        self.exp
    }

    fn get_jti(&self) -> &str {
        &self.jti
    }
}

fn is_role_admin(roles: &str) -> Result<(), AuthError> {
    if !security::roles::is_role_admin(roles) {
        return Err(AuthError::WrongCredentials);
    }
    Ok(())
}

#[async_trait]
impl<S> FromRequestParts<S> for AccessClaims
where
    SharedState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        decode_token(parts, state).await
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RefreshClaims
where
    SharedState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        decode_token(parts, state).await
    }
}

async fn decode_token<S, T>(parts: &mut Parts, state: &S) -> Result<T, AuthError>
where
    SharedState: FromRef<S>,
    S: Send + Sync,
    T: for<'de> serde::Deserialize<'de> + std::fmt::Debug + ClaimsMethods,
{
    // extract the token from the authorization header
    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| {
            tracing::error!("Invalid authorization header");
            AuthError::WrongCredentials
        })?;

    // decode the user data
    let token_data = jsonwebtoken::decode::<T>(
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
