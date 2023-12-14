use sqlx::postgres::PgPoolOptions;
use std::{collections::HashMap, sync::Arc};
use tokio::signal;
use tower_http::cors::CorsLayer;

use hyper::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderMap, Method, StatusCode,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axum::{
    extract::{Query, Request, State},
    http::{self, HeaderValue},
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

    tracing::info!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    // parse configuration
    let config = config::from_dotenv();

    // connect to redis
    let redis = match redis::Client::open(config.redis_url()) {
        Ok(redis) => {
            tracing::info!("Connected to redis");
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
            tracing::info!("Connected to postgres");
            pool
        }
        Err(e) => {
            tracing::error!("Could not connect to postgres: {}", e);
            std::process::exit(1);
        }
    };

    // run migrations
    sqlx::migrate!("db/migrations").run(&pgpool).await.unwrap();

    // build a CORS layer
    let cors_header_value = config.service_http_addr().parse::<HeaderValue>().unwrap();
    let cors_layer = CorsLayer::new()
        .allow_origin(cors_header_value)
        .allow_methods([
            Method::HEAD,
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    // get the listening address
    let addr = config.service_socket_addr();

    // build the state
    let shared_state = Arc::new(AppState {
        pgpool,
        redis,
        config,
    });

    // build the app
    let app = routes(shared_state).layer(cors_layer);

    // build the listener
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", addr);

    // start the service
    axum::serve(listener, app).await.unwrap();

    /*
       // hyper v1 => shutdown requires bilerplate logic now :(
       // run the hyper service
       hyper::Server::bind(&addr)
       .serve(routes.into_make_service())
       .with_graceful_shutdown(shutdown_signal())
       .await
       .unwrap();
    */

    tracing::info!("server shutdown successfully.");
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
    tracing::debug!("entered: root_handler()");
    Html("<h1>Axum-Web</h1>")
}

async fn head_request_handler(State(_state): State<SharedState>, method: http::Method) -> Response {
    tracing::debug!("entered head_request_handler()");
    // it usually only makes sense to special-case HEAD
    // if computing the body has some relevant cost
    if method == http::Method::HEAD {
        tracing::debug!("head method found");
        return ([("x-some-header", "header from HEAD")]).into_response();
    }

    ([("x-some-header", "header from GET")], "body from GET").into_response()
}

async fn any_request_handler(
    State(_state): State<SharedState>,
    method: http::Method,
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

async fn _shutdown_signal() {
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

    tracing::info!("received termination signal, shutting down...");
}
