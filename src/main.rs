use sqlx::postgres::PgPoolOptions;
use std::{collections::HashMap, sync::Arc};
use tokio::signal;

use hyper::{Body, HeaderMap, Request, StatusCode};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axum::{
    extract::{Query, State},
    http,
    response::{Html, IntoResponse, Response},
    routing::{any, get},
    Router,
};

use axum_web::{
    auth, config,
    state::{AppState, SharedState},
    user,
};

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

    // parse configuration
    let config = config::from_dotenv();

    // connect to redis
    let redis = match redis::Client::open(config.redis_url()) {
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
    let pgpool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.postgres_url())
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

    // get service listening address
    let addr = config.service_addr();
    if tracing::enabled!(Level::DEBUG) {
        tracing::debug!("listening on {}", addr);
    }

    // build the state
    let shared_state = Arc::new(AppState {
        pgpool,
        redis,
        config,
    });

    // build the service routes
    let routes = routes(shared_state);

    // run the hyper service
    hyper::Server::bind(&addr)
        .serve(routes.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    if tracing::enabled!(Level::INFO) {
        tracing::info!("server shutdown successfully.");
    }
}

fn routes(state: SharedState) -> Router {
    // build the service routes
    Router::new()
        // add a fallback service for handling routes to unknown paths
        .fallback(error_404_handler)
        .route("/", get(root_handler))
        .route("/head", get(head_request_handler))
        .route("/any", any(any_request_handler))
        // nesting the authentication related routes under `/auth`
        .nest("/auth", auth::routes())
        // nesting the user related routes under `/user`
        .nest("/user", user::routes())
        .with_state(state)
}

async fn root_handler(State(_state): State<SharedState>) -> Html<&'static str> {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("entered: root_handler()");
    }
    Html("<h1>Axum-Web</h1>")
}

async fn head_request_handler(State(_state): State<SharedState>, method: http::Method) -> Response {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("entered head_request_handler()");
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

async fn any_request_handler(
    State(_state): State<SharedState>,
    method: http::Method,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    request: Request<Body>,
) -> Response {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("entered: any_request_handler()");
        tracing::trace!("method: {:?}", method);
        tracing::trace!("headers: {:?}", headers);
        tracing::trace!("params: {:?}", params);
        tracing::trace!("request: {:?}", request);
    }

    (StatusCode::OK, "any").into_response()
}

async fn error_404_handler(State(_state): State<SharedState>) -> impl IntoResponse {
    if tracing::enabled!(Level::TRACE) {
        tracing::trace!("entered: error_404_handler()");
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
