use axum::{
    Router,
    routing::{any, delete, get, post},
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// module imports
use cirith::admin::{
    create_api_key, create_route, delete_api_key, delete_route, list_api_keys, list_routes,
};
use cirith::auth::AuthValidator;
use cirith::config::{AppState, Config};
use cirith::metrics::{Metrics, metrics_handler};
use cirith::proxy::proxy_handler;
use cirith::rate_limit::RateLimiter;
use cirith::storage::Database;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::load("config.yml").expect("Failed to load config");
    let port = config.server.port;
    tracing::info!("Loaded {} routes", config.routes.len());

    let database = Arc::new(
        Database::new(&config.database.url)
            .await
            .expect("Failed to connect to database"),
    );

    let http_client = reqwest::Client::builder()
        .pool_max_idle_per_host(50)
        .pool_idle_timeout(Duration::from_secs(90))
        .timeout(Duration::from_secs(config.server.timeout_seconds))
        .build()
        .expect("Failed to create HTTP client");

    let rate_limiter = RateLimiter::new(
        config.rate_limit.max_requests,
        config.rate_limit.window_secs,
    );

    let metrics = Arc::new(Metrics::new());
    let auth_validator = AuthValidator::new(&config.auth);
    let state = Arc::new(AppState {
        config,
        http_client,
        rate_limiter,
        auth_validator,
        metrics,
        database,
    });

    let rate_limiter = Arc::new(state.rate_limiter.clone());
    rate_limiter.spawn_cleanup_task(300);
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .route("/admin/routes", get(list_routes))
        .route("/admin/routes", post(create_route))
        .route("/admin/routes/{*path}", delete(delete_route))
        .route("/admin/keys", get(list_api_keys))
        .route("/admin/keys", post(create_api_key))
        .route("/admin/keys/{name}", delete(delete_api_key))
        .route("/{*path}", any(proxy_handler))
        .with_state(state)
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}
