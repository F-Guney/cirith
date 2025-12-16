mod handlers;
mod metrics;
mod state;

use axum::{
    Router,
    routing::{delete, get, post},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// Imports
use crate::handlers::{
    health::{health_check, metrics_handler},
    keys::{create_api_key, delete_api_key, list_api_keys},
    routes::{create_route, delete_route, list_routes},
};
use crate::metrics::Metrics;
use crate::state::AdminState;
use cirith_shared::{auth::AuthValidator, config::Config, storage::Database};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::load("config.yml").expect("Failed to load config.yml");
    let port = config.server.admin_port;

    let database = Arc::new(
        Database::new(&config.database.url)
            .await
            .expect("Failed to connect to database"),
    );

    let metrics = Arc::new(Metrics::new());
    let auth_validator = AuthValidator::new(&config.auth);
    let state = Arc::new(AdminState {
        config,
        database,
        metrics,
        auth_validator,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .route("/admin/routes", get(list_routes))
        .route("/admin/routes", post(create_route))
        .route("/admin/routes/{*path}", delete(delete_route))
        .route("/admin/keys", get(list_api_keys))
        .route("/admin/keys", post(create_api_key))
        .route("/admin/keys/{name}", delete(delete_api_key))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
