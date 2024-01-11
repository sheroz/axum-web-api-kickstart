use axum::{
    body::Body,
    extract::{Path, Query, Request},
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{any, get},
    Json, Router,
};
use serde_json::json;
use std::collections::HashMap;

use super::{auth, users};

use crate::application::{
    api_error::ApiError,
    api_version::{self, ApiVersion},
    app_const::*,
    security::jwt_claims::AccessClaims,
    state::SharedState,
};

pub fn routes(state: SharedState) -> Router {
    // build the service routes
    Router::new()
        .route("/", get(root_handler))
        .route("/head", get(head_request_handler))
        .route("/any", any(any_request_handler))
        .route("/:version/heartbeat/:id", get(heartbeat_handler))
        // nesting the authentication related routes
        .nest("/:version/auth", auth::routes())
        // nesting the user related routes
        .nest("/:version/users", users::routes())
        // add a fallback service for handling routes to unknown paths
        .fallback(error_404_handler)
        .with_state(state)
}

#[tracing::instrument(level = tracing::Level::TRACE, name = "axum", skip_all, fields(method=request.method().to_string(), uri=request.uri().to_string()))]
pub async fn logging_middleware(request: Request<Body>, next: Next) -> Response {
    tracing::trace!(
        "received a {} request to {}",
        request.method(),
        request.uri()
    );
    next.run(request).await
}

async fn heartbeat_handler(
    Path((version, id)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let api_version: ApiVersion = api_version::parse_version(&version)?;
    tracing::trace!("heartbeat: api version: {}", api_version);
    tracing::trace!("heartbeat: received id: {}", id);
    let map = HashMap::from([
        ("service".to_string(), SERVICE_NAME.to_string()),
        ("version".to_string(), SERVICE_VERSION.to_string()),
        ("heartbeat-id".to_string(), id),
    ]);
    Ok(Json(map))
}

async fn root_handler(access_claims: AccessClaims) -> Result<impl IntoResponse, ApiError> {
    if tracing::enabled!(tracing::Level::TRACE) {
        tracing::trace!(
            "current timestamp, chrono::Utc {}",
            chrono::Utc::now().timestamp() as usize
        );
        let start = std::time::SystemTime::now();
        let validation_timestamp = start
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        tracing::trace!("current timestamp, std::time {}", validation_timestamp);
        tracing::trace!("authentication details: {:#?}", access_claims);
    }
    Ok(Json(json!({"message": "Hello from Axum-Web!"})))
}

async fn head_request_handler(method: Method) -> Response {
    // it usually only makes sense to special-case HEAD
    // if computing the body has some relevant cost
    if method == Method::HEAD {
        tracing::debug!("HEAD method found");
        return [("x-some-header", "header from HEAD")].into_response();
    }

    ([("x-some-header", "header from GET")], "body from GET").into_response()
}

async fn any_request_handler(
    method: Method,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    request: Request,
) -> impl IntoResponse {
    if tracing::enabled!(tracing::Level::DEBUG) {
        tracing::debug!("method: {:?}", method);
        tracing::debug!("headers: {:?}", headers);
        tracing::debug!("params: {:?}", params);
        tracing::debug!("request: {:?}", request);
    }

    StatusCode::OK
}

async fn error_404_handler(request: Request) -> impl IntoResponse {
    tracing::error!("route not found: {:?}", request);
    StatusCode::NOT_FOUND
}
