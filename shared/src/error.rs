use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Route not found")]
    RouteNotFound,

    #[error("Upstream request failed: {0}")]
    UpstreamRequest(String),

    #[error("Unsupported method")]
    UnsupportedMethod,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Config error: {0}")]
    Config(String),
}
