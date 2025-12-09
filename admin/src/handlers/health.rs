use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::collections::HashMap;
use std::sync::{Arc, atomic::Ordering};
// imports
use crate::state::AdminState;

pub async fn metrics_handler(State(state): State<Arc<AdminState>>) -> impl IntoResponse {
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

pub async fn health_check() -> &'static str {
    "OK"
}
