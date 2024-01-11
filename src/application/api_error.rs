use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct ApiError {
    pub status_code: StatusCode,
    pub error_message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{status_code: {}, error_message: {}}}",
            self.status_code, self.error_message
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::error!("Error response: {}", self.to_string());
        (self.status_code, self.error_message).into_response()
    }
}

impl From<StatusCode> for ApiError {
    fn from(status_code: StatusCode) -> Self {
        ApiError {
            status_code,
            error_message: status_code.to_string(),
        }
    }
}
