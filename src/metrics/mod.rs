use crate::config::AppState;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct Metrics {
    pub total_requests: AtomicU64,
    pub successful_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    pub rate_limited_requests: AtomicU64,
    pub unauthorized_requests: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_total(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_successful(&self) {
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_failed(&self) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_rate_limited(&self) {
        self.rate_limited_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_unauthorized(&self) {
        self.unauthorized_requests.fetch_add(1, Ordering::Relaxed);
    }
}

pub async fn metrics_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let total = state.metrics.total_requests.load(Ordering::Relaxed);
    let successful = state.metrics.successful_requests.load(Ordering::Relaxed);
    let failed = state.metrics.failed_requests.load(Ordering::Relaxed);
    let rate_limited = state.metrics.rate_limited_requests.load(Ordering::Relaxed);
    let unauthorized = state.metrics.unauthorized_requests.load(Ordering::Relaxed);

    let result: HashMap<String, u64> = HashMap::from([
        ("total".to_string(), total),
        ("successful".to_string(), successful),
        ("failed".to_string(), failed),
        ("rate-limited".to_string(), rate_limited),
        ("unauthorized".to_string(), unauthorized),
    ]);

    (StatusCode::OK, Json(result)).into_response()
}
