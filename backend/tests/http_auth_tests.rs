mod common;
mod http_common;

use axum::body::Bytes;
use http_common::{http_ctx, HttpTestContext};
use poker_tracker::models::user::{AuthResponse, User};
use rstest::rstest;
use serde_json::json;

// =============================================================================
// Phase 2: Health Check & Basic Routing Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_health_endpoint_returns_ok(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let response = ctx.server.get("/api/health").await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["status"], "Ok");
}

#[rstest]
#[tokio::test]
async fn test_unknown_route_without_auth_returns_401(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    // Unknown routes go through auth middleware first, returning 401
    let response = ctx.server.get("/api/nonexistent").await;
    response.assert_status_unauthorized();
}

#[rstest]
#[tokio::test]
async fn test_wrong_method_on_health_returns_405(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let response = ctx.server.post("/api/health").await;
    response.assert_status(axum::http::StatusCode::METHOD_NOT_ALLOWED);
}

// =============================================================================
// Phase 3: Public Auth Endpoint Tests (Register & Login)
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_register_with_valid_data_returns_token(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let response = ctx
        .server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test@example.com",
            "username": "testuser",
            "password": "password123"
        }))
        .await;

    response.assert_status(axum::http::StatusCode::CREATED);
    let body: AuthResponse = response.json();
    assert!(!body.token.is_empty());
    assert_eq!(body.user.email, "test@example.com");
    assert_eq!(body.user.username, "testuser");
}

#[rstest]
#[tokio::test]
async fn test_register_invalid_email_returns_400(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let response = ctx
        .server
        .post("/api/auth/register")
        .json(&json!({
            "email": "not-an-email",
            "username": "testuser",
            "password": "password123"
        }))
        .await;

    response.assert_status_bad_request();
}

#[rstest]
#[tokio::test]
async fn test_register_short_password_returns_400(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let response = ctx
        .server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test@example.com",
            "username": "testuser",
            "password": "short"
        }))
        .await;

    response.assert_status_bad_request();
}

#[rstest]
#[tokio::test]
async fn test_register_short_username_returns_400(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let response = ctx
        .server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test@example.com",
            "username": "ab",
            "password": "password123"
        }))
        .await;

    response.assert_status_bad_request();
}

#[rstest]
#[tokio::test]
async fn test_register_duplicate_email_returns_409(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // First registration
    ctx.server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test@example.com",
            "username": "testuser1",
            "password": "password123"
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Duplicate email
    let response = ctx
        .server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test@example.com",
            "username": "testuser2",
            "password": "password123"
        }))
        .await;

    response.assert_status(axum::http::StatusCode::CONFLICT);
}

#[rstest]
#[tokio::test]
async fn test_register_duplicate_username_returns_409(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // First registration
    ctx.server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test1@example.com",
            "username": "testuser",
            "password": "password123"
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Duplicate username
    let response = ctx
        .server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test2@example.com",
            "username": "testuser",
            "password": "password123"
        }))
        .await;

    response.assert_status(axum::http::StatusCode::CONFLICT);
}

#[rstest]
#[tokio::test]
async fn test_login_with_valid_credentials_returns_token(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // First register
    ctx.server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test@example.com",
            "username": "testuser",
            "password": "password123"
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Then login
    let response = ctx
        .server
        .post("/api/auth/login")
        .json(&json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .await;

    response.assert_status_ok();
    let body: AuthResponse = response.json();
    assert!(!body.token.is_empty());
    assert_eq!(body.user.email, "test@example.com");
}

#[rstest]
#[tokio::test]
async fn test_login_wrong_password_returns_401(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // First register
    ctx.server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test@example.com",
            "username": "testuser",
            "password": "password123"
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Try login with wrong password
    let response = ctx
        .server
        .post("/api/auth/login")
        .json(&json!({
            "email": "test@example.com",
            "password": "wrongpassword"
        }))
        .await;

    response.assert_status_unauthorized();
}

#[rstest]
#[tokio::test]
async fn test_login_nonexistent_user_returns_401(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    let response = ctx
        .server
        .post("/api/auth/login")
        .json(&json!({
            "email": "nonexistent@example.com",
            "password": "password123"
        }))
        .await;

    response.assert_status_unauthorized();
}

#[rstest]
#[tokio::test]
async fn test_malformed_json_returns_400(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    let response = ctx
        .server
        .post("/api/auth/register")
        .content_type("application/json")
        .bytes(Bytes::from_static(b"{ invalid json }"))
        .await;

    // Axum returns 400 for malformed JSON
    response.assert_status_bad_request();
}

#[rstest]
#[tokio::test]
async fn test_missing_required_fields_returns_422(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    let response = ctx
        .server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test@example.com"
            // missing username and password
        }))
        .await;

    response.assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
}

// =============================================================================
// Phase 4: Protected Endpoints & Auth Middleware Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_protected_endpoint_without_token_returns_401(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let response = ctx.server.get("/api/auth/me").await;
    response.assert_status_unauthorized();
}

#[rstest]
#[tokio::test]
async fn test_protected_endpoint_with_invalid_token_returns_401(
    #[future] http_ctx: HttpTestContext,
) {
    let ctx = http_ctx.await;
    let response = ctx
        .server
        .get("/api/auth/me")
        .add_header("Authorization", "Bearer invalid-token")
        .await;
    response.assert_status_unauthorized();
}

#[rstest]
#[tokio::test]
async fn test_protected_endpoint_with_malformed_auth_header_returns_401(
    #[future] http_ctx: HttpTestContext,
) {
    let ctx = http_ctx.await;

    // Missing "Bearer " prefix
    let response = ctx
        .server
        .get("/api/auth/me")
        .add_header("Authorization", "just-a-token")
        .await;
    response.assert_status_unauthorized();
}

#[rstest]
#[tokio::test]
async fn test_get_me_with_valid_token_returns_user(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // Register to get token
    let register_response = ctx
        .server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test@example.com",
            "username": "testuser",
            "password": "password123"
        }))
        .await;
    let auth: AuthResponse = register_response.json();

    // Access protected endpoint
    let response = ctx
        .server
        .get("/api/auth/me")
        .add_header("Authorization", format!("Bearer {}", auth.token))
        .await;

    response.assert_status_ok();
    let user: User = response.json();
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.username, "testuser");
}

#[rstest]
#[tokio::test]
async fn test_update_cookie_consent_with_valid_token(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // Register
    let register_response = ctx
        .server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test@example.com",
            "username": "testuser",
            "password": "password123"
        }))
        .await;
    let auth: AuthResponse = register_response.json();

    // Verify initial cookie_consent is false
    assert!(!auth.user.cookie_consent);

    // Update cookie consent
    let response = ctx
        .server
        .put("/api/auth/cookie-consent")
        .add_header("Authorization", format!("Bearer {}", auth.token))
        .json(&json!({ "cookie_consent": true }))
        .await;

    response.assert_status_ok();

    // Verify update via /me endpoint
    let me_response = ctx
        .server
        .get("/api/auth/me")
        .add_header("Authorization", format!("Bearer {}", auth.token))
        .await;
    let user: User = me_response.json();
    assert!(user.cookie_consent);
}

#[rstest]
#[tokio::test]
async fn test_change_password_with_valid_credentials(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // Register
    let register_response = ctx
        .server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test@example.com",
            "username": "testuser",
            "password": "password123"
        }))
        .await;
    let auth: AuthResponse = register_response.json();

    // Change password
    let response = ctx
        .server
        .post("/api/auth/change-password")
        .add_header("Authorization", format!("Bearer {}", auth.token))
        .json(&json!({
            "old_password": "password123",
            "new_password": "newpassword456"
        }))
        .await;

    response.assert_status_ok();

    // Verify can login with new password
    let login_response = ctx
        .server
        .post("/api/auth/login")
        .json(&json!({
            "email": "test@example.com",
            "password": "newpassword456"
        }))
        .await;
    login_response.assert_status_ok();
}

#[rstest]
#[tokio::test]
async fn test_change_password_wrong_old_password_returns_401(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // Register
    let register_response = ctx
        .server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test@example.com",
            "username": "testuser",
            "password": "password123"
        }))
        .await;
    let auth: AuthResponse = register_response.json();

    // Try to change password with wrong old password
    let response = ctx
        .server
        .post("/api/auth/change-password")
        .add_header("Authorization", format!("Bearer {}", auth.token))
        .json(&json!({
            "old_password": "wrongpassword",
            "new_password": "newpassword456"
        }))
        .await;

    response.assert_status_unauthorized();
}

#[rstest]
#[tokio::test]
async fn test_change_password_short_new_password_returns_400(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // Register
    let register_response = ctx
        .server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test@example.com",
            "username": "testuser",
            "password": "password123"
        }))
        .await;
    let auth: AuthResponse = register_response.json();

    // Try to change to a short password
    let response = ctx
        .server
        .post("/api/auth/change-password")
        .add_header("Authorization", format!("Bearer {}", auth.token))
        .json(&json!({
            "old_password": "password123",
            "new_password": "short"
        }))
        .await;

    response.assert_status_bad_request();
}

#[rstest]
#[tokio::test]
async fn test_sessions_endpoint_without_token_returns_401(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let response = ctx.server.get("/api/sessions").await;
    response.assert_status_unauthorized();
}

#[rstest]
#[tokio::test]
async fn test_token_from_login_works_for_protected_endpoints(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // Register
    ctx.server
        .post("/api/auth/register")
        .json(&json!({
            "email": "test@example.com",
            "username": "testuser",
            "password": "password123"
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Login to get fresh token
    let login_response = ctx
        .server
        .post("/api/auth/login")
        .json(&json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .await;
    let auth: AuthResponse = login_response.json();

    // Use login token for protected endpoint
    let response = ctx
        .server
        .get("/api/auth/me")
        .add_header("Authorization", format!("Bearer {}", auth.token))
        .await;

    response.assert_status_ok();
    let user: User = response.json();
    assert_eq!(user.email, "test@example.com");
}
