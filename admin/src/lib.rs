use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{delete, get, post},
};
use std::sync::Arc;
// imports
use crate::handlers::health::{health_check, metrics_handler};
use crate::handlers::keys::{create_api_key, delete_api_key, list_api_keys};
use crate::handlers::routes::{create_route, delete_route, list_routes};
use crate::state::AdminState;

pub mod handlers;
pub mod metrics;
pub mod middleware;
pub mod state;

pub fn create_app(state: Arc<AdminState>) -> Router {
    let public_routes = Router::new().route("/health", get(health_check));

    let protected_routes = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/admin/routes", get(list_routes))
        .route("/admin/routes", post(create_route))
        .route("/admin/routes/{*path}", delete(delete_route))
        .route("/admin/keys", get(list_api_keys))
        .route("/admin/keys", post(create_api_key))
        .route("/admin/keys/{name}", delete(delete_api_key))
        .layer(from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
}
