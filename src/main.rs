use axum::{
    extract::Query,
    http,
    response::{Html, IntoResponse, Response},
    routing::{any, get, post},
    Router,
};
use hyper::{Body, HeaderMap, Request, StatusCode};
use sqlx::postgres::PgPoolOptions;
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
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    if tracing::enabled!(Level::INFO) {
        tracing::info!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    }

    // connect to redis
    let redis_url = "redis://127.0.0.1:6379";
    let _redis = match redis::Client::open(redis_url) {
        Ok(redis) => {
            if tracing::enabled!(tracing::Level::INFO) {
                tracing::info!("Connected to redis");
            }
            redis
        }
        Err(e) => {
            tracing::error!("Could not connect to redis: {}", e);
            std::process::exit(1);
        }
    };

    // connect to postgres
    let postgres_url = "postgresql://admin:pswd1234@localhost:5432/axum_web";
    let pgpool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(postgres_url)
        .await
    {
        Ok(pool) => {
            if tracing::enabled!(tracing::Level::INFO) {
                tracing::info!("Connected to postgres");
            }
            pool
        }
        Err(e) => {
            tracing::error!("Could not connect to postgres: {}", e);
            std::process::exit(1);
        }
    };

    // run migrations
    sqlx::migrate!("db/migrations").run(&pgpool).await.unwrap();

    // build application with a router
    let app = build_router();

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

fn build_router() -> Router {
    // build our application with a route
    Router::new()
        // add a fallback service for handling routes to unknown paths
        .fallback(error_404_handler)
        .route("/", get(root_handler))
        .route("/head", get(head_request_handler))
        .route("/login", post(login_handler_post).get(login_handler_get))
        .route("/any", any(any_request_handler))
}

async fn root_handler() -> Html<&'static str> {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("entered: handler_root()");
    }
    Html("axum-web")
}

async fn head_request_handler(method: http::Method) -> Response {
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

async fn login_handler_get() -> Response {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("entered: handler_get_login()");
    }
    (StatusCode::FORBIDDEN, "forbidden").into_response()
}

async fn login_handler_post() -> Response {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("entered: handler_post_login()");
    }
    (StatusCode::FORBIDDEN, "forbidden").into_response()
}

async fn any_request_handler(
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

async fn error_404_handler() -> impl IntoResponse {
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
