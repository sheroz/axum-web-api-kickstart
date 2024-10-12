use axum::{
    async_trait,
    extract::{FromRequestParts, Path},
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use std::collections::HashMap;

use super::api_error::ApiError;

#[derive(Debug, Clone, Copy)]
pub enum ApiVersion {
    V1,
    V2,
}

impl std::str::FromStr for ApiVersion {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v1" => Ok(ApiVersion::V1),
            "v2" => Ok(ApiVersion::V2),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v: &str = match self {
            ApiVersion::V1 => "v1",
            ApiVersion::V2 => "v2",
        };
        write!(f, "{}", v)
    }
}

pub fn parse_version(version: &str) -> Result<ApiVersion, ApiError> {
    match version.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err(ApiVersionError::InvalidApiVersion(format!(
            "Unknown API Version: {}",
            version
        ))
        .into()),
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for ApiVersion
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let params: Path<HashMap<String, String>> = parts
            .extract()
            .await
            .map_err(|_| ApiVersionError::VersionExtractError)?;

        let version = params
            .get("version")
            .ok_or(ApiVersionError::ParameterMissing)?;

        parse_version(version)
    }
}

#[derive(Debug)]
pub enum ApiVersionError {
    InvalidApiVersion(String),
    ParameterMissing,
    VersionExtractError,
}

impl From<ApiVersionError> for ApiError {
    fn from(err: ApiVersionError) -> Self {
        let (status_code, error_message) = match err {
            ApiVersionError::InvalidApiVersion(error_message) => {
                (StatusCode::NOT_ACCEPTABLE, error_message)
            }
            ApiVersionError::ParameterMissing => (
                StatusCode::NOT_ACCEPTABLE,
                "parameter is missing: version".to_owned(),
            ),
            ApiVersionError::VersionExtractError => (
                StatusCode::BAD_REQUEST,
                "Could not extract api version".to_owned(),
            ),
        };
        ApiError {
            status_code,
            error_message,
        }
    }
}
