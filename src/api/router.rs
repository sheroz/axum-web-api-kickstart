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

use super::{
    auth::{self, JwtClaims},
    users,
};
use crate::shared::state::SharedState;

pub fn routes(state: SharedState) -> Router {
    // build the service routes
    Router::new()
        .route("/", get(root_handler))
        .route("/heartbeat/:id", get(heartbeat_handler))
        .route("/head", get(head_request_handler))
        .route("/any", any(any_request_handler))
        // nesting the authentication related routes under `/auth`
        .nest("/auth", auth::routes())
        // nesting the user related routes under `/user`
        .nest("/users", users::routes())
        // add a fallback service for handling routes to unknown paths
        .fallback(error_404_handler)
        .with_state(state)
}

pub async fn logging_middleware(request: Request<Body>, next: Next) -> Response {
    tracing::trace!(
        "Received a {} request to {}",
        request.method(),
        request.uri()
    );
    next.run(request).await
}

async fn heartbeat_handler(Path(id): Path<String>) -> impl IntoResponse {
    let map = HashMap::from([
        ("service".to_string(), "axum-web".to_string()),
        ("heartbeat-id".to_string(), id),
    ]);
    Json(map)
}

async fn root_handler(claims: JwtClaims) -> impl IntoResponse {
    tracing::debug!("entered: root_handler()");
    tracing::trace!("login details: {:#?}", claims);
    Json(json!({"message": "Hello from Axum-Web!"}))
}

async fn head_request_handler(method: Method) -> Response {
    tracing::debug!("entered head_request_handler()");
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
        tracing::debug!("entered: any_request_handler()");
        tracing::debug!("method: {:?}", method);
        tracing::debug!("headers: {:?}", headers);
        tracing::debug!("params: {:?}", params);
        tracing::debug!("request: {:?}", request);
    }

    StatusCode::OK
}

async fn error_404_handler(request: Request) -> impl IntoResponse {
    tracing::debug!("entered: error_404_handler()");
    tracing::error!("route not found: {:?}", request);
    StatusCode::NOT_FOUND
}
