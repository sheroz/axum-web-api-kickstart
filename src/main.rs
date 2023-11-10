use axum::{
    extract::Query,
    http,
    response::{Html, IntoResponse, Response},
    routing::{any, get, post},
    Router,
};
use hyper::{Body, HeaderMap, Request, StatusCode};
use std::{collections::HashMap, net::SocketAddr};
use tokio::signal;
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_web=trace".into()),
                .unwrap_or_else(|_| "axum_web=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    if tracing::enabled!(Level::INFO) {
        tracing::info!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    }

    if tracing::enabled!(Level::INFO) {
        tracing::info!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    }

    // build our application with a route
    let app = Router::new()
        .route("/", get(handler_root))
        .route("/head", get(handler_head))
        .route("/login", post(handler_post_login).get(handler_get_login))
        .route("/any", any(handler_any));

    // add a fallback service for handling routes to unknown paths
    let app = app.fallback(handler_404);

    // run the hyper service
    // run the hyper service
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    if tracing::enabled!(Level::DEBUG) {
        tracing::debug!("listening on {}", addr);
    }


    if tracing::enabled!(Level::DEBUG) {
        tracing::debug!("listening on {}", addr);
    }

    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    if tracing::enabled!(Level::INFO) {
        tracing::info!("server shutdown successfully.");
    }
}

async fn handler_root() -> Html<&'static str> {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("entered: handler_root()");
    }
    Html("axum-web")
}

async fn handler_head(method: http::Method) -> Response {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("entered handler_head()");
    }
    // it usually only makes sense to special-case HEAD
    // if computing the body has some relevant cost
    if method == http::Method::HEAD {
        if tracing::enabled!(Level::TRACE) {
            tracing::trace!("head method found");
        }

        return ([("x-some-header", "header from HEAD")]).into_response();
    }

    ([("x-some-header", "header from GET")], "body from GET").into_response()
}

async fn handler_get_login() -> Response {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("entered: handler_get_login()");
    }
    (StatusCode::FORBIDDEN, "forbidden").into_response()
}

async fn handler_post_login() -> Response {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("entered: handler_post_login()");
    }
    (StatusCode::FORBIDDEN, "forbidden").into_response()
}

async fn handler_any(
    method: http::Method,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    request: Request<Body>,
) -> Response {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("entered: handler_any()");
        tracing::trace!("method: {:?}", method);
        tracing::trace!("headers: {:?}", headers);
        tracing::trace!("params: {:?}", params);
        tracing::trace!("request: {:?}", request);
    }

    (StatusCode::OK, "any").into_response()
}

async fn handler_404() -> impl IntoResponse {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("entered: handler_404()");
    }
    (StatusCode::NOT_FOUND, "not found")
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    if tracing::enabled!(Level::INFO) {
        tracing::info!("received termination signal, shutting down...");
    }
}
