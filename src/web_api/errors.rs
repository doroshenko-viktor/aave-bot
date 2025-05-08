use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use eyre::Report;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Not found")]
    NotFound,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, msg.to_string())
            }
            ApiError::InternalServerError(msg) => {
                #[cfg(feature = "dev")]
                {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        msg.to_string(),
                    )
                }
                #[cfg(not(feature = "dev"))]
                {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An unexpected error occurred".to_string(),
                    )
                }
            }
        };

        (status, body).into_response()
    }
}

impl From<Report> for ApiError {
    fn from(report: Report) -> Self {
        ApiError::InternalServerError(report.to_string())
    }
}
