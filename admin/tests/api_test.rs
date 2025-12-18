use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;
// imports
use cirith_admin::create_app;
use cirith_admin::metrics::Metrics;
use cirith_admin::state::AdminState;
use cirith_shared::auth::AuthValidator;
use cirith_shared::config::{
    AdminConfig, AuthConfig, Config, DatabaseConfig, RateLimitConfig, ServerConfig,
};
use cirith_shared::storage::Database;

async fn setup_test_app() -> axum::Router {
    let database = Database::new(":memory:").await.unwrap();
    let config = Config {
        server: ServerConfig {
            admin_port: 3000,
            gateway_port: 6191,
            timeout_seconds: 30,
        },
        auth: AuthConfig {
            enabled: false,
            api_keys: vec![],
        },
        rate_limit: RateLimitConfig {
            max_requests: 100,
            window_secs: 60,
        },
        database: DatabaseConfig {
            url: ":memory:".to_string(),
        },
        admin: AdminConfig {
            token: "test-token".to_string(),
        },
    };

    let auth_validator = AuthValidator::new(&config.auth);
    let metrics = Arc::new(Metrics::new());
    let state = Arc::new(AdminState {
        config,
        auth_validator,
        metrics,
        database: Arc::new(database),
    });

    create_app(state)
}

#[tokio::test]
async fn test_health_returns_200() {
    let app = setup_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_routes_without_token_returns_401() {
    let app = setup_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/routes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_routes_with_token_returns_200() {
    let app = setup_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/routes")
                .header("Authorization", "Bearer test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
