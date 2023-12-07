use axum::{
    extract::State,
    response::{IntoResponse, Response},
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

async fn login_handler(State(_state): State<SharedState>) -> Response {
    tracing::debug!("entered: handler_get_login()");
    (StatusCode::FORBIDDEN, "forbidden").into_response()
}

async fn logout_handler(State(_state): State<SharedState>) -> Response {
    tracing::debug!("entered: handler_post_login()");
    (StatusCode::FORBIDDEN, "forbidden").into_response()
}
