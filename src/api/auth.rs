use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use hyper::StatusCode;

use crate::shared::state::SharedState;

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/logout", get(logout_handler))
}

async fn login_handler(State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: login_handler()");
    StatusCode::FORBIDDEN
}

async fn logout_handler(State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: logout_handler()");
    StatusCode::FORBIDDEN
}
