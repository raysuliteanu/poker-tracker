#![allow(dead_code)]

use axum_test::TestServer;
use poker_tracker::app::{AppState, create_app_router};
use poker_tracker::models::user::AuthResponse;
use rstest::fixture;
use serde_json::json;
use std::sync::Arc;

use crate::common::PooledConnectionTestDb;

/// Test context combining axum-test server with testcontainers database
pub struct HttpTestContext {
    pub server: TestServer,
    _db_provider: Arc<PooledConnectionTestDb>, // Keep TestDb alive for the container
}

impl HttpTestContext {
    pub async fn new() -> Self {
        // Set JWT_SECRET for tests
        unsafe {
            std::env::set_var("JWT_SECRET", "test_secret_key_for_http_testing");
        }

        let db_provider = Arc::new(PooledConnectionTestDb::new().await);
        let app_state = Arc::new(AppState {
            db_provider: db_provider.clone() as Arc<dyn poker_tracker::utils::DbProvider>,
        });
        let router = create_app_router(app_state);
        let server = TestServer::new(router).expect("Failed to create test server");

        Self {
            server,
            _db_provider: db_provider,
        }
    }
}

#[fixture]
pub async fn http_ctx() -> HttpTestContext {
    HttpTestContext::new().await
}

/// Register a user and return the JWT token
pub async fn register_and_get_token(ctx: &HttpTestContext, email: &str) -> String {
    let username = email.split('@').next().unwrap();
    let response = ctx
        .server
        .post("/api/auth/register")
        .json(&json!({
            "email": email,
            "username": username,
            "password": "password123"
        }))
        .await;
    let auth: AuthResponse = response.json();
    auth.token
}

/// Default session request JSON for testing
pub fn default_session_json() -> serde_json::Value {
    json!({
        "session_date": "2024-01-15",
        "duration_minutes": 120,
        "buy_in_amount": 100.0,
        "rebuy_amount": 0.0,
        "cash_out_amount": 150.0
    })
}
