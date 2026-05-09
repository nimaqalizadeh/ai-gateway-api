use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

#[allow(dead_code)] // TODO(phase-5): remove once handlers start returning AppError
pub type AppResult<T> = Result<T, AppError>;

#[derive(Serialize)]
#[allow(dead_code)] // TODO(phase-5): remove once handlers start returning AppError
struct ErrorBody {
    error: String,
    code: &'static str,
}

#[derive(Debug, Error)]
#[allow(dead_code)] // TODO(phase-5): remove once handlers start returning AppError
pub enum AppError {
    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error), // unexpected/bug - 500

    #[error("{0}")]
    BadRequest(String), // client send bad input - 400

    #[error("unauthorized")]
    Unauthorized, // missing or invalid auth - 401

    #[error("rate-limited")]
    RateLimited, // too many requests - 429

    #[error("upstream error: {0}, {1}")]
    Upstream(StatusCode, String), // LLM server fails

    #[error("not found")]
    NotFound, // unknown route or resource - 404
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::Internal(err) => {
                // TODO tracing::error!
                eprintln!("internal server error: {err:?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    "internal server error".to_string(),
                )
            }

            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad-request", msg),

            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "missing or invalid token".to_string(),
            ),

            AppError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate-limited",
                "too many requests".to_string(),
            ),

            AppError::Upstream(status_code, upstream_msg) => {
                // TODO tracing::error!
                eprintln!("Upstream error: Status Code: {status_code} - Message: {upstream_msg}");
                (
                    StatusCode::BAD_GATEWAY,
                    "upstream-error",
                    "Provider is not available now".to_string(),
                )
            }

            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "not-found",
                "the resource not found".to_string(),
            ),
        };
        let body = ErrorBody {
            error: message,
            code,
        };
        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unauthorized_into_response_has_correct_shape() {
        let response = AppError::Unauthorized.into_response();
        let status = response.status();
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(json["code"], "unauthorized");
        assert_eq!(json["error"], "missing or invalid token");
    }
}
