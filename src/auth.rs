use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use hyper::StatusCode;

use crate::state::SharedState;

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/logout", get(logout_handler))
}

async fn login_handler(State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: handler_get_login()");
    StatusCode::FORBIDDEN
}

async fn logout_handler(State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: handler_post_login()");
    StatusCode::FORBIDDEN
}
