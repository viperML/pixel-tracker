use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Error type which implements `IntoResponse`
#[derive(Debug)]
pub struct AppError(pub eyre::Report);

impl<E: Into<eyre::Report>> From<E> for AppError {
    fn from(error: E) -> Self {
        AppError(error.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
    }
}

// Result type
pub type AppResult<T, E = AppError> = std::result::Result<T, E>;
