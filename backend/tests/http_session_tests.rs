mod common;
mod http_common;

use axum::http::StatusCode;
use http_common::{HttpTestContext, default_session_json, http_ctx, register_and_get_token};
use poker_tracker::models::poker_session::SessionWithProfit;
use poker_tracker::models::user::AuthResponse;
use rstest::rstest;
use serde_json::json;

// =============================================================================
// Phase 5: Session CRUD HTTP Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_create_session_with_valid_data(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    let response = ctx
        .server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "session_date": "2024-01-15",
            "duration_minutes": 120,
            "buy_in_amount": 100.0,
            "cash_out_amount": 150.0
        }))
        .await;

    response.assert_status(StatusCode::CREATED);
    let session: SessionWithProfit = response.json();
    assert_eq!(session.profit, 50.0);
    assert_eq!(session.session.duration_minutes, 120);
}

#[rstest]
#[tokio::test]
async fn test_create_session_with_rebuy(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    let response = ctx
        .server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "session_date": "2024-01-15",
            "duration_minutes": 180,
            "buy_in_amount": 100.0,
            "rebuy_amount": 50.0,
            "cash_out_amount": 200.0
        }))
        .await;

    response.assert_status(StatusCode::CREATED);
    let session: SessionWithProfit = response.json();
    // profit = 200 - (100 + 50) = 50
    assert_eq!(session.profit, 50.0);
}

#[rstest]
#[tokio::test]
async fn test_create_session_with_notes(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    let response = ctx
        .server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "session_date": "2024-01-15",
            "duration_minutes": 60,
            "buy_in_amount": 100.0,
            "cash_out_amount": 80.0,
            "notes": "Bad session, tilted on river"
        }))
        .await;

    response.assert_status(StatusCode::CREATED);
    let session: SessionWithProfit = response.json();
    assert_eq!(
        session.session.notes,
        Some("Bad session, tilted on river".to_string())
    );
    assert_eq!(session.profit, -20.0);
}

#[rstest]
#[tokio::test]
async fn test_create_session_invalid_date_returns_400(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    let response = ctx
        .server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "session_date": "invalid-date",
            "duration_minutes": 120,
            "buy_in_amount": 100.0,
            "cash_out_amount": 150.0
        }))
        .await;

    response.assert_status_bad_request();
}

#[rstest]
#[tokio::test]
async fn test_create_session_zero_duration_returns_400(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    let response = ctx
        .server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "session_date": "2024-01-15",
            "duration_minutes": 0,
            "buy_in_amount": 100.0,
            "cash_out_amount": 150.0
        }))
        .await;

    response.assert_status_bad_request();
}

#[rstest]
#[tokio::test]
async fn test_get_sessions_empty(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    let response = ctx
        .server
        .get("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_ok();
    let sessions: Vec<SessionWithProfit> = response.json();
    assert!(sessions.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_get_sessions_returns_multiple(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    // Create 3 sessions
    for i in 1..=3 {
        ctx.server
            .post("/api/sessions")
            .add_header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "session_date": format!("2024-01-{:02}", i),
                "duration_minutes": 60 * i,
                "buy_in_amount": 100.0,
                "cash_out_amount": 100.0 + (i as f64 * 10.0)
            }))
            .await
            .assert_status(StatusCode::CREATED);
    }

    let response = ctx
        .server
        .get("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_ok();
    let sessions: Vec<SessionWithProfit> = response.json();
    assert_eq!(sessions.len(), 3);
}

#[rstest]
#[tokio::test]
async fn test_get_sessions_user_isolation(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // User 1 creates a session
    let token1 = register_and_get_token(&ctx, "user1@example.com").await;
    ctx.server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token1))
        .json(&default_session_json())
        .await
        .assert_status(StatusCode::CREATED);

    // User 2 should not see User 1's session
    let token2 = register_and_get_token(&ctx, "user2@example.com").await;
    let response = ctx
        .server
        .get("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token2))
        .await;

    response.assert_status_ok();
    let sessions: Vec<SessionWithProfit> = response.json();
    assert!(sessions.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_get_session_by_id(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    // Create session
    let create_response = ctx
        .server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&default_session_json())
        .await;
    let created: SessionWithProfit = create_response.json();

    // Get by ID
    let response = ctx
        .server
        .get(&format!("/api/sessions/{}", created.session.id))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_ok();
    let session: SessionWithProfit = response.json();
    assert_eq!(session.session.id, created.session.id);
}

#[rstest]
#[tokio::test]
async fn test_get_session_not_found_returns_404(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    let response = ctx
        .server
        .get(&format!("/api/sessions/{}", uuid::Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_not_found();
}

#[rstest]
#[tokio::test]
async fn test_get_session_wrong_user_returns_404(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // User 1 creates session
    let token1 = register_and_get_token(&ctx, "user1@example.com").await;
    let create_response = ctx
        .server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token1))
        .json(&default_session_json())
        .await;
    let created: SessionWithProfit = create_response.json();

    // User 2 tries to get it
    let token2 = register_and_get_token(&ctx, "user2@example.com").await;
    let response = ctx
        .server
        .get(&format!("/api/sessions/{}", created.session.id))
        .add_header("Authorization", format!("Bearer {}", token2))
        .await;

    response.assert_status_not_found();
}

#[rstest]
#[tokio::test]
async fn test_update_session_all_fields(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    // Create session
    let create_response = ctx
        .server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&default_session_json())
        .await;
    let created: SessionWithProfit = create_response.json();

    // Update all fields
    let response = ctx
        .server
        .put(&format!("/api/sessions/{}", created.session.id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "session_date": "2024-02-20",
            "duration_minutes": 240,
            "buy_in_amount": 200.0,
            "rebuy_amount": 100.0,
            "cash_out_amount": 500.0,
            "notes": "Updated notes"
        }))
        .await;

    response.assert_status_ok();
    let updated: SessionWithProfit = response.json();
    assert_eq!(updated.session.duration_minutes, 240);
    assert_eq!(updated.profit, 200.0); // 500 - (200 + 100)
    assert_eq!(updated.session.notes, Some("Updated notes".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_update_session_partial(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    // Create session
    let create_response = ctx
        .server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "session_date": "2024-01-15",
            "duration_minutes": 120,
            "buy_in_amount": 100.0,
            "cash_out_amount": 150.0,
            "notes": "Original notes"
        }))
        .await;
    let created: SessionWithProfit = create_response.json();

    // Update only notes
    let response = ctx
        .server
        .put(&format!("/api/sessions/{}", created.session.id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "notes": "Updated notes only"
        }))
        .await;

    response.assert_status_ok();
    let updated: SessionWithProfit = response.json();
    // Original values preserved
    assert_eq!(updated.session.duration_minutes, 120);
    assert_eq!(updated.profit, 50.0);
    // Only notes changed
    assert_eq!(
        updated.session.notes,
        Some("Updated notes only".to_string())
    );
}

#[rstest]
#[tokio::test]
async fn test_update_session_wrong_user_returns_404(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // User 1 creates session
    let token1 = register_and_get_token(&ctx, "user1@example.com").await;
    let create_response = ctx
        .server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token1))
        .json(&default_session_json())
        .await;
    let created: SessionWithProfit = create_response.json();

    // User 2 tries to update it
    let token2 = register_and_get_token(&ctx, "user2@example.com").await;
    let response = ctx
        .server
        .put(&format!("/api/sessions/{}", created.session.id))
        .add_header("Authorization", format!("Bearer {}", token2))
        .json(&json!({ "notes": "Hacked!" }))
        .await;

    response.assert_status_not_found();
}

#[rstest]
#[tokio::test]
async fn test_delete_session_success(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    // Create session
    let create_response = ctx
        .server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&default_session_json())
        .await;
    let created: SessionWithProfit = create_response.json();

    // Delete
    let response = ctx
        .server
        .delete(&format!("/api/sessions/{}", created.session.id))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_ok();

    // Verify deleted
    let get_response = ctx
        .server
        .get(&format!("/api/sessions/{}", created.session.id))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    get_response.assert_status_not_found();
}

#[rstest]
#[tokio::test]
async fn test_delete_session_wrong_user_returns_404(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // User 1 creates session
    let token1 = register_and_get_token(&ctx, "user1@example.com").await;
    let create_response = ctx
        .server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token1))
        .json(&default_session_json())
        .await;
    let created: SessionWithProfit = create_response.json();

    // User 2 tries to delete it
    let token2 = register_and_get_token(&ctx, "user2@example.com").await;
    let response = ctx
        .server
        .delete(&format!("/api/sessions/{}", created.session.id))
        .add_header("Authorization", format!("Bearer {}", token2))
        .await;

    response.assert_status_not_found();

    // Verify still exists for User 1
    let get_response = ctx
        .server
        .get(&format!("/api/sessions/{}", created.session.id))
        .add_header("Authorization", format!("Bearer {}", token1))
        .await;

    get_response.assert_status_ok();
}

#[rstest]
#[tokio::test]
async fn test_delete_session_not_found_returns_404(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    let response = ctx
        .server
        .delete(&format!("/api/sessions/{}", uuid::Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_not_found();
}

// =============================================================================
// Phase 6: CSV Export & Edge Cases
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_export_sessions_csv_content_type(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    let response = ctx
        .server
        .get("/api/sessions/export")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_ok();
    // Check content-type header
    let header = response.header("content-type");
    let content_type = header.to_str().unwrap();
    assert!(
        content_type.contains("text/csv"),
        "Expected text/csv, got: {}",
        content_type
    );
}

#[rstest]
#[tokio::test]
async fn test_export_sessions_csv_has_header(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    let response = ctx
        .server
        .get("/api/sessions/export")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_ok();
    let csv = response.text();
    assert!(csv.contains("Date,Duration (hours),Buy-in,Rebuy,Cash Out,Profit/Loss,Notes"));
}

#[rstest]
#[tokio::test]
async fn test_export_sessions_csv_contains_data(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    // Create a session
    ctx.server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "session_date": "2024-03-15",
            "duration_minutes": 120,
            "buy_in_amount": 100.0,
            "cash_out_amount": 175.0,
            "notes": "Test session for CSV"
        }))
        .await
        .assert_status(StatusCode::CREATED);

    let response = ctx
        .server
        .get("/api/sessions/export")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_ok();
    let csv = response.text();
    assert!(csv.contains("2024-03-15"));
    assert!(csv.contains("2.0")); // 120 minutes = 2.0 hours
    assert!(csv.contains("75.00")); // profit
    assert!(csv.contains("Test session for CSV"));
}

#[rstest]
#[tokio::test]
async fn test_export_sessions_csv_escapes_special_chars(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    // Create a session with special characters in notes
    ctx.server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "session_date": "2024-03-15",
            "duration_minutes": 60,
            "buy_in_amount": 100.0,
            "cash_out_amount": 100.0,
            "notes": "Notes with, comma and \"quotes\""
        }))
        .await
        .assert_status(StatusCode::CREATED);

    let response = ctx
        .server
        .get("/api/sessions/export")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_ok();
    let csv = response.text();
    // Should be properly escaped
    assert!(csv.contains("\"Notes with, comma and \"\"quotes\"\"\""));
}

#[rstest]
#[tokio::test]
async fn test_export_sessions_empty_returns_header_only(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    let response = ctx
        .server
        .get("/api/sessions/export")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_ok();
    let csv = response.text();
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 1); // Just the header
}

// =============================================================================
// Phase 7: Full Workflow Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_complete_user_workflow(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // 1. Register
    let register_resp = ctx
        .server
        .post("/api/auth/register")
        .json(&json!({
            "email": "poker@example.com",
            "username": "pokerplayer",
            "password": "securepwd123"
        }))
        .await;
    register_resp.assert_status(axum::http::StatusCode::CREATED);
    let auth: AuthResponse = register_resp.json();
    let token = auth.token;

    // 2. Get profile
    let me_resp = ctx
        .server
        .get("/api/auth/me")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;
    me_resp.assert_status_ok();

    // 3. Update cookie consent
    ctx.server
        .put("/api/auth/cookie-consent")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({ "cookie_consent": true }))
        .await
        .assert_status_ok();

    // 4. Create multiple sessions
    for i in 1..=3 {
        ctx.server
            .post("/api/sessions")
            .add_header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "session_date": format!("2024-01-{:02}", i),
                "duration_minutes": 60 * i,
                "buy_in_amount": 100.0,
                "cash_out_amount": 100.0 + (i as f64 * 25.0)
            }))
            .await
            .assert_status(StatusCode::CREATED);
    }

    // 5. Get all sessions
    let sessions_resp = ctx
        .server
        .get("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;
    let sessions: Vec<SessionWithProfit> = sessions_resp.json();
    assert_eq!(sessions.len(), 3);

    // 6. Export CSV
    let export_resp = ctx
        .server
        .get("/api/sessions/export")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;
    export_resp.assert_status_ok();
    let csv = export_resp.text();
    assert!(csv.contains("2024-01-01"));
    assert!(csv.contains("2024-01-03"));

    // 7. Change password
    ctx.server
        .post("/api/auth/change-password")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "old_password": "securepwd123",
            "new_password": "newsecurepwd456"
        }))
        .await
        .assert_status_ok();

    // 8. Login with new password
    let login_resp = ctx
        .server
        .post("/api/auth/login")
        .json(&json!({
            "email": "poker@example.com",
            "password": "newsecurepwd456"
        }))
        .await;
    login_resp.assert_status_ok();
}

#[rstest]
#[tokio::test]
async fn test_multi_user_isolation_workflow(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;

    // User 1 registers and creates sessions
    let token1 = register_and_get_token(&ctx, "user1@example.com").await;
    for i in 1..=2 {
        ctx.server
            .post("/api/sessions")
            .add_header("Authorization", format!("Bearer {}", token1))
            .json(&json!({
                "session_date": format!("2024-01-{:02}", i),
                "duration_minutes": 60,
                "buy_in_amount": 100.0,
                "cash_out_amount": 150.0
            }))
            .await
            .assert_status(StatusCode::CREATED);
    }

    // User 2 registers and creates sessions
    let token2 = register_and_get_token(&ctx, "user2@example.com").await;
    for i in 3..=5 {
        ctx.server
            .post("/api/sessions")
            .add_header("Authorization", format!("Bearer {}", token2))
            .json(&json!({
                "session_date": format!("2024-01-{:02}", i),
                "duration_minutes": 90,
                "buy_in_amount": 200.0,
                "cash_out_amount": 180.0
            }))
            .await
            .assert_status(StatusCode::CREATED);
    }

    // User 1 sees only their 2 sessions
    let user1_sessions: Vec<SessionWithProfit> = ctx
        .server
        .get("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token1))
        .await
        .json();
    assert_eq!(user1_sessions.len(), 2);
    assert!(user1_sessions.iter().all(|s| s.profit == 50.0));

    // User 2 sees only their 3 sessions
    let user2_sessions: Vec<SessionWithProfit> = ctx
        .server
        .get("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token2))
        .await
        .json();
    assert_eq!(user2_sessions.len(), 3);
    assert!(user2_sessions.iter().all(|s| s.profit == -20.0));

    // User 1's export has 2 data rows
    let export1 = ctx
        .server
        .get("/api/sessions/export")
        .add_header("Authorization", format!("Bearer {}", token1))
        .await
        .text();
    assert_eq!(export1.lines().count(), 3); // header + 2 rows

    // User 2's export has 3 data rows
    let export2 = ctx
        .server
        .get("/api/sessions/export")
        .add_header("Authorization", format!("Bearer {}", token2))
        .await
        .text();
    assert_eq!(export2.lines().count(), 4); // header + 3 rows
}

#[rstest]
#[tokio::test]
async fn test_session_crud_lifecycle(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let token = register_and_get_token(&ctx, "test@example.com").await;

    // Create
    let create_resp = ctx
        .server
        .post("/api/sessions")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "session_date": "2024-06-15",
            "duration_minutes": 180,
            "buy_in_amount": 500.0,
            "cash_out_amount": 750.0,
            "notes": "Initial notes"
        }))
        .await;
    create_resp.assert_status(StatusCode::CREATED);
    let session: SessionWithProfit = create_resp.json();
    let session_id = session.session.id;
    assert_eq!(session.profit, 250.0);

    // Read
    let read_resp = ctx
        .server
        .get(&format!("/api/sessions/{}", session_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;
    read_resp.assert_status_ok();
    let read_session: SessionWithProfit = read_resp.json();
    assert_eq!(read_session.session.id, session_id);

    // Update
    let update_resp = ctx
        .server
        .put(&format!("/api/sessions/{}", session_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "cash_out_amount": 1000.0,
            "notes": "Updated: big win!"
        }))
        .await;
    update_resp.assert_status_ok();
    let updated_session: SessionWithProfit = update_resp.json();
    assert_eq!(updated_session.profit, 500.0);
    assert_eq!(
        updated_session.session.notes,
        Some("Updated: big win!".to_string())
    );

    // Delete
    ctx.server
        .delete(&format!("/api/sessions/{}", session_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .await
        .assert_status_ok();

    // Verify deleted
    ctx.server
        .get(&format!("/api/sessions/{}", session_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .await
        .assert_status_not_found();
}
