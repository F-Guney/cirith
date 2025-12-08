use axum::{
    http::status::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Upstream request failed: {0}")]
    UpstreamRequest(#[from] reqwest::Error),

    #[error("Upstream timeout")]
    UpstreamTimeout,

    #[error("Method not supported: {0}")]
    UnsupportedMethod(String),

    #[error("Route not found: {0}")]
    RouteNotFound(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Unauthorized")]
    Unauthorized,
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::UpstreamRequest(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            Self::UpstreamTimeout => (StatusCode::GATEWAY_TIMEOUT, self.to_string()),
            Self::UnsupportedMethod(_) => (StatusCode::METHOD_NOT_ALLOWED, self.to_string()),
            Self::RouteNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            Self::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
        };

        tracing::error!("{}", message);
        (status, message).into_response()
    }
}
