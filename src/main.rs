use axum::{
    response::{Html, IntoResponse},
    routing::{any, get},
    Router,
};
use hyper::StatusCode;
use std::net::SocketAddr;
use tokio::signal;
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_web=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    if tracing::enabled!(Level::INFO) {
        tracing::info!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    }

    // build our application with a route
    let app = Router::new()
        .route("/", get(handler_root))
        .route("/login", any(handler_login));

    // add a fallback service for handling routes to unknown paths
    let app = app.fallback(handler_404);

    // run the hyper service
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

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
        tracing::trace!("request: /");
    }
    Html("axum-web")
}

async fn handler_login() -> impl IntoResponse {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("request: login");
    }
    (StatusCode::FORBIDDEN, "forbidden")
}

async fn handler_404() -> impl IntoResponse {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("request: unknown route");
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
