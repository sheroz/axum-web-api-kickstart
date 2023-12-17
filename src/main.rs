use sqlx::postgres::PgPoolOptions;
use std::{collections::HashMap, sync::Arc};
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};

use hyper::{
    HeaderMap, Method, StatusCode,
};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use axum::{
    extract::{Query, Request, State},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{any, get}, Router, Json,
};
use axum::body::Body;
use axum::extract::Path;

use sqlx::{Pool, Postgres};

use axum_web::{
    auth, config::{self, Config},
    state::{AppState, SharedState},
    users,
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
    let redis = connect_redis(&config).await;

    // connect to postgres
    let pgpool = connect_postgres(&config).await;

    // run migrations
    sqlx::migrate!("db/migrations").run(&pgpool).await.unwrap();

    // build a CORS layer
    let cors_layer = CorsLayer::new().allow_origin(Any);
    // let cors_header_value = config.service_http_addr().parse::<HeaderValue>().unwrap();
    // let cors_layer = CorsLayer::new()
    //      .allow_origin(cors_header_value)
    //      .allow_methods([
    //          Method::HEAD,
    //          Method::GET,
    //          Method::POST,
    //          Method::PATCH,
    //          Method::DELETE,
    //      ])
    //      .allow_credentials(true)
    //      .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    // get the listening address
    let addr = config.service_socket_addr();

    // build the state
    let shared_state = Arc::new(AppState {
        pgpool,
        redis,
        config,
    });

    // build the app
    let app = routes(shared_state)
        .layer(cors_layer)
        .layer(middleware::from_fn(logging_middleware));
    // build the listener
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", addr);

    // start the service
    axum::serve(listener, app).await.unwrap();

    /*
       // hyper v1 => shutdown requires boilerplate logic now :(
       // run the hyper service
       hyper::Server::bind(&addr)
       .serve(routes.into_make_service())
       .with_graceful_shutdown(shutdown_signal())
       .await
       .unwrap();
    */

    tracing::info!("server shutdown successfully.");
}

async fn connect_redis(config: &Config) -> redis::aio::Connection {
    match redis::Client::open(config.redis_url()) {
        Ok(redis) => {
            match redis.get_async_connection().await {
                Ok(connection) => {
                    tracing::info!("Connected to redis");
                    connection
                }
                Err(e) => {
                    tracing::error!("Could not connect to redis: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            tracing::error!("Could not open redis: {}", e);
            std::process::exit(1);
        }
    }
}

async fn connect_postgres(config: &Config) -> Pool<Postgres> {
    match PgPoolOptions::new()
        .max_connections(config.postgres_connection_pool)
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
    }
}

fn routes(state: SharedState) -> Router {
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

async fn logging_middleware(request: Request<Body>, next: Next) -> Response {
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
