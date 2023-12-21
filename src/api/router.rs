use std::collections::HashMap;
use axum::{
    middleware::Next,
    http::{HeaderMap, Method, StatusCode},
    body::Body,
    Json,
    Router,
    response::{Html, IntoResponse, Response},
    extract::{Path, Query, Request, State},
    routing::{any, get},
};
use super::{auth, users};
use crate::shared::state::SharedState;

pub fn routes(state: SharedState) -> Router {
    // build the service routes
    Router::new()
        // add a fallback service for handling routes to unknown paths
        .fallback(error_404_handler)
        .route("/", get(root_handler))
        .route("/heartbeat/:id", get(heartbeat_handler))
        .route("/head", get(head_request_handler))
        .route("/any", any(any_request_handler))
        // nesting the authentication related routes under `/auth`
        .nest("/auth", auth::routes())
        // nesting the user related routes under `/user`
        .nest("/users", users::routes())
        .with_state(state)
}

pub async fn logging_middleware(request: Request<Body>, next: Next) -> Response {
    tracing::trace!("Received a {} request to {}", request.method(), request.uri());
    next.run(request).await
}

async fn heartbeat_handler(Path(id): Path<String>) -> impl IntoResponse {
    let map = HashMap::from([
        ("service".to_string(), "axum-web".to_string()),
        ("heartbeat-id".to_string(), id)]);
    Json(map)
}

async fn root_handler(State(_state): State<SharedState>) -> Html<&'static str> {
    tracing::debug!("entered: root_handler()");
    Html("<h1>Axum-Web</h1>")
}

async fn head_request_handler(State(_state): State<SharedState>, method: Method) -> Response {
    tracing::debug!("entered head_request_handler()");
    // it usually only makes sense to special-case HEAD
    // if computing the body has some relevant cost
    if method == Method::HEAD {
        tracing::debug!("head method found");
        return [("x-some-header", "header from HEAD")].into_response();
    }

    ([("x-some-header", "header from GET")], "body from GET").into_response()
}

async fn any_request_handler(
    State(_state): State<SharedState>,
    method: Method,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    request: Request,
) -> Response {
    if tracing::enabled!(tracing::Level::DEBUG) {
        tracing::debug!("entered: any_request_handler()");
        tracing::debug!("method: {:?}", method);
        tracing::debug!("headers: {:?}", headers);
        tracing::debug!("params: {:?}", params);
        tracing::debug!("request: {:?}", request);
    }

    (StatusCode::OK, "any").into_response()
}

async fn error_404_handler(State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: error_404_handler()");
    (StatusCode::NOT_FOUND, "not found")
}
