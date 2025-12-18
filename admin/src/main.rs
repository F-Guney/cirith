use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// Imports
use cirith_admin::{create_app, metrics::Metrics, state::AdminState};
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

    let app = create_app(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
