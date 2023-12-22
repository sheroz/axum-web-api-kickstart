use axum_web::{
    api::router,
    infrastructure::{postgres, redis},
    shared::{config, state::AppState},
};
use std::sync::Arc;
use tokio::{signal, sync::Mutex};
use tower_http::cors::{Any, CorsLayer};
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

    tracing::info!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    // parse configuration
    let config = config::from_dotenv();

    // connect to redis
    let redis = redis::open(&config).await;

    // connect to postgres
    let pgpool = postgres::pgpool(&config).await;

    // run migrations
    sqlx::migrate!("src/infrastructure/postgres/migrations")
        .run(&pgpool)
        .await
        .unwrap();

    // build a CORS layer
    // see https://docs.rs/tower-http/latest/tower_http/cors/index.html
    // for more details
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
        redis: Mutex::new(redis),
        config,
    });

    // build the app
    let app = router::routes(shared_state)
        .layer(cors_layer)
        .layer(axum::middleware::from_fn(router::logging_middleware));

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
