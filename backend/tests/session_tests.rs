mod common;

use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use common::{
    TestDb, create_test_user_raw, default_session_request, get_session_by_id, get_sessions_for_user,
};
use diesel::{prelude::*, sql_types::Integer};
use poker_tracker::handlers::poker_session::{
    self, CreateSessionError, DeleteSessionError, GetSessionError, UpdateSessionError,
};
use poker_tracker::models::{
    CreatePokerSessionRequest, UpdatePokerSessionRequest, calculate_profit,
};
use poker_tracker::utils::DbConnectionProvider;
use rstest::rstest;
use uuid::Uuid;

use crate::common::fixtures::test_db;

use poker_tracker::models::user::{NewUser, User};
use poker_tracker::schema::users;

// =============================================================================
// Database Connection Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_database_connection(#[future] test_db: TestDb) {
    let db = test_db.await;
    let mut conn = db.get_connection().expect("Failed to get db connection");
    let result = diesel::select(diesel::dsl::sql::<Integer>("1")).execute(&mut conn);
    assert!(result.is_ok());
}

// =============================================================================
// HIGH PRIORITY: Create Session Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_create_session(#[future] test_db: TestDb) {
    let db = test_db.await;

    // Create a test user using the db connection provider
    let mut conn = db.get_connection().expect("Failed to get db connection");
    let new_user = NewUser {
        email: "test@test.com".to_string(),
        username: "test".to_string(),
        password_hash: "1234".to_string(),
    };

    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(&mut conn)
        .expect("Failed to create test user");

    // Create a session request
    let session_req = CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 120,
        buy_in_amount: 100.0,
        rebuy_amount: Some(50.0),
        cash_out_amount: 200.0,
        notes: Some("Test session".to_string()),
    };

    // Call the handler using the TestDb as the connection provider
    let session = poker_session::do_create_session(&db, user.id, session_req)
        .await
        .expect("Failed to create session");

    // Verify the session was created correctly
    assert_eq!(session.user_id, user.id);
    assert_eq!(session.duration_minutes, 120);
    assert_eq!(session.notes, Some("Test session".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_create_session_minimal(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Create a session with only required fields (no rebuy, no notes)
    let session_req = CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 60,
        buy_in_amount: 100.0,
        rebuy_amount: None,
        cash_out_amount: 150.0,
        notes: None,
    };

    let session = poker_session::do_create_session(&db, user.id, session_req)
        .await
        .expect("Failed to create session");

    assert_eq!(session.user_id, user.id);
    assert_eq!(session.duration_minutes, 60);
    assert_eq!(session.rebuy_amount, BigDecimal::from_f64(0.0).unwrap());
    assert!(session.notes.is_none());
}

#[rstest]
#[tokio::test]
async fn test_create_session_with_rebuy(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    let session_req = CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 180,
        buy_in_amount: 200.0,
        rebuy_amount: Some(100.0),
        cash_out_amount: 500.0,
        notes: None,
    };

    let session = poker_session::do_create_session(&db, user.id, session_req)
        .await
        .expect("Failed to create session");

    assert_eq!(session.rebuy_amount, BigDecimal::from_f64(100.0).unwrap());
}

#[rstest]
#[tokio::test]
async fn test_create_session_with_notes(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    let session_req = CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 120,
        buy_in_amount: 100.0,
        rebuy_amount: None,
        cash_out_amount: 150.0,
        notes: Some("Great session at the casino!".to_string()),
    };

    let session = poker_session::do_create_session(&db, user.id, session_req)
        .await
        .expect("Failed to create session");

    assert_eq!(
        session.notes,
        Some("Great session at the casino!".to_string())
    );
}

#[rstest]
#[tokio::test]
async fn test_create_session_invalid_date_format(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Invalid date format (MM/DD/YYYY instead of YYYY-MM-DD)
    let session_req = CreatePokerSessionRequest {
        session_date: "01/15/2024".to_string(),
        duration_minutes: 120,
        buy_in_amount: 100.0,
        rebuy_amount: None,
        cash_out_amount: 150.0,
        notes: None,
    };

    let result = poker_session::do_create_session(&db, user.id, session_req).await;

    assert!(matches!(
        result,
        Err(CreateSessionError::InvalidDateFormat(_))
    ));
}

#[rstest]
#[tokio::test]
async fn test_create_session_generates_uuid(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    let session = poker_session::do_create_session(&db, user.id, default_session_request())
        .await
        .expect("Failed to create session");

    // Session should have a valid, non-nil UUID
    assert!(!session.id.is_nil());
}

#[rstest]
#[tokio::test]
async fn test_create_session_persists_to_database(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    let session = poker_session::do_create_session(&db, user.id, default_session_request())
        .await
        .expect("Failed to create session");

    // Verify we can retrieve the session from the database
    let retrieved = get_session_by_id(&db, session.id);
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, session.id);
    assert_eq!(retrieved.user_id, user.id);
}

// =============================================================================
// HIGH PRIORITY: Get Sessions Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_get_sessions_empty(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // User with no sessions should return empty array
    let sessions = get_sessions_for_user(&db, user.id);
    assert!(sessions.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_get_sessions_multiple(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Create multiple sessions
    for i in 1..=3 {
        let session_req = CreatePokerSessionRequest {
            session_date: format!("2024-01-{:02}", i),
            duration_minutes: 60 * i,
            buy_in_amount: 100.0,
            rebuy_amount: None,
            cash_out_amount: 150.0,
            notes: Some(format!("Session {}", i)),
        };
        poker_session::do_create_session(&db, user.id, session_req)
            .await
            .expect("Failed to create session");
    }

    let sessions = get_sessions_for_user(&db, user.id);
    assert_eq!(sessions.len(), 3);

    // Sessions should be ordered by date descending
    assert_eq!(sessions[0].notes, Some("Session 3".to_string()));
    assert_eq!(sessions[1].notes, Some("Session 2".to_string()));
    assert_eq!(sessions[2].notes, Some("Session 1".to_string()));
}

// =============================================================================
// HIGH PRIORITY: User Isolation Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_get_sessions_user_isolation(#[future] test_db: TestDb) {
    let db = test_db.await;

    // Create two users
    let user_a = create_test_user_raw(&db, "usera@test.com", "usera");
    let user_b = create_test_user_raw(&db, "userb@test.com", "userb");

    // Create sessions for user A
    let session_req_a = CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 120,
        buy_in_amount: 100.0,
        rebuy_amount: None,
        cash_out_amount: 200.0,
        notes: Some("User A session".to_string()),
    };
    poker_session::do_create_session(&db, user_a.id, session_req_a)
        .await
        .expect("Failed to create session");

    // Create sessions for user B
    let session_req_b = CreatePokerSessionRequest {
        session_date: "2024-01-16".to_string(),
        duration_minutes: 180,
        buy_in_amount: 200.0,
        rebuy_amount: Some(50.0),
        cash_out_amount: 300.0,
        notes: Some("User B session".to_string()),
    };
    poker_session::do_create_session(&db, user_b.id, session_req_b)
        .await
        .expect("Failed to create session");

    // User A should only see their own sessions
    let sessions_a = get_sessions_for_user(&db, user_a.id);
    assert_eq!(sessions_a.len(), 1);
    assert_eq!(sessions_a[0].notes, Some("User A session".to_string()));

    // User B should only see their own sessions
    let sessions_b = get_sessions_for_user(&db, user_b.id);
    assert_eq!(sessions_b.len(), 1);
    assert_eq!(sessions_b[0].notes, Some("User B session".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_create_session_assigns_correct_user(#[future] test_db: TestDb) {
    let db = test_db.await;

    let user_a = create_test_user_raw(&db, "usera@test.com", "usera");
    let user_b = create_test_user_raw(&db, "userb@test.com", "userb");

    // Create session for user A
    let session = poker_session::do_create_session(&db, user_a.id, default_session_request())
        .await
        .expect("Failed to create session");

    // Session should belong to user A, not user B
    assert_eq!(session.user_id, user_a.id);
    assert_ne!(session.user_id, user_b.id);
}

// =============================================================================
// HIGH PRIORITY: Profit Calculation Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_profit_calculation_positive(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Buy in: 100, No rebuy, Cash out: 200 = Profit: 100
    let session_req = CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 120,
        buy_in_amount: 100.0,
        rebuy_amount: None,
        cash_out_amount: 200.0,
        notes: None,
    };

    let session = poker_session::do_create_session(&db, user.id, session_req)
        .await
        .expect("Failed to create session");

    let profit = calculate_profit(
        &session.buy_in_amount,
        &session.rebuy_amount,
        &session.cash_out_amount,
    );

    assert!((profit - 100.0).abs() < 0.01);
}

#[rstest]
#[tokio::test]
async fn test_profit_calculation_negative(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Buy in: 200, Rebuy: 100, Cash out: 150 = Profit: -150
    let session_req = CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 120,
        buy_in_amount: 200.0,
        rebuy_amount: Some(100.0),
        cash_out_amount: 150.0,
        notes: None,
    };

    let session = poker_session::do_create_session(&db, user.id, session_req)
        .await
        .expect("Failed to create session");

    let profit = calculate_profit(
        &session.buy_in_amount,
        &session.rebuy_amount,
        &session.cash_out_amount,
    );

    assert!((profit - (-150.0)).abs() < 0.01);
}

#[rstest]
#[tokio::test]
async fn test_profit_calculation_break_even(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Buy in: 100, No rebuy, Cash out: 100 = Profit: 0
    let session_req = CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 120,
        buy_in_amount: 100.0,
        rebuy_amount: None,
        cash_out_amount: 100.0,
        notes: None,
    };

    let session = poker_session::do_create_session(&db, user.id, session_req)
        .await
        .expect("Failed to create session");

    let profit = calculate_profit(
        &session.buy_in_amount,
        &session.rebuy_amount,
        &session.cash_out_amount,
    );

    assert!((profit - 0.0).abs() < 0.01);
}

#[rstest]
#[tokio::test]
async fn test_profit_calculation_with_rebuy(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Buy in: 100, Rebuy: 50, Cash out: 250 = Profit: 100
    let session_req = CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 120,
        buy_in_amount: 100.0,
        rebuy_amount: Some(50.0),
        cash_out_amount: 250.0,
        notes: None,
    };

    let session = poker_session::do_create_session(&db, user.id, session_req)
        .await
        .expect("Failed to create session");

    let profit = calculate_profit(
        &session.buy_in_amount,
        &session.rebuy_amount,
        &session.cash_out_amount,
    );

    assert!((profit - 100.0).abs() < 0.01);
}

#[rstest]
#[tokio::test]
async fn test_profit_calculation_decimal_precision(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Buy in: 99.99, Rebuy: 50.01, Cash out: 175.50 = Profit: 25.50
    let session_req = CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 120,
        buy_in_amount: 99.99,
        rebuy_amount: Some(50.01),
        cash_out_amount: 175.50,
        notes: None,
    };

    let session = poker_session::do_create_session(&db, user.id, session_req)
        .await
        .expect("Failed to create session");

    let profit = calculate_profit(
        &session.buy_in_amount,
        &session.rebuy_amount,
        &session.cash_out_amount,
    );

    assert!((profit - 25.50).abs() < 0.01);
}

#[rstest]
#[tokio::test]
async fn test_amounts_stored_correctly(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    let session_req = CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 120,
        buy_in_amount: 123.45,
        rebuy_amount: Some(67.89),
        cash_out_amount: 234.56,
        notes: None,
    };

    let session = poker_session::do_create_session(&db, user.id, session_req)
        .await
        .expect("Failed to create session");

    // Verify amounts are stored correctly
    let buy_in: f64 = session.buy_in_amount.to_f64().unwrap();
    let rebuy: f64 = session.rebuy_amount.to_f64().unwrap();
    let cash_out: f64 = session.cash_out_amount.to_f64().unwrap();

    assert!((buy_in - 123.45).abs() < 0.01);
    assert!((rebuy - 67.89).abs() < 0.01);
    assert!((cash_out - 234.56).abs() < 0.01);
}

// =============================================================================
// MEDIUM PRIORITY: Get Session Tests (Not Found / Authorization)
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_get_session_success(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Create a session
    let created = poker_session::do_create_session(&db, user.id, default_session_request())
        .await
        .expect("Failed to create session");

    // Get the session
    let retrieved =
        poker_session::do_get_session(&db, created.id, user.id).expect("Failed to get session");

    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.user_id, user.id);
}

#[rstest]
#[tokio::test]
async fn test_get_session_not_found(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Try to get a non-existent session
    let fake_session_id = Uuid::new_v4();
    let result = poker_session::do_get_session(&db, fake_session_id, user.id);

    assert!(matches!(result, Err(GetSessionError::NotFound)));
}

#[rstest]
#[tokio::test]
async fn test_get_session_wrong_user(#[future] test_db: TestDb) {
    let db = test_db.await;

    let user_a = create_test_user_raw(&db, "usera@test.com", "usera");
    let user_b = create_test_user_raw(&db, "userb@test.com", "userb");

    // Create a session for user A
    let session = poker_session::do_create_session(&db, user_a.id, default_session_request())
        .await
        .expect("Failed to create session");

    // User B tries to get user A's session - should fail with NotFound
    let result = poker_session::do_get_session(&db, session.id, user_b.id);

    assert!(matches!(result, Err(GetSessionError::NotFound)));
}

// =============================================================================
// MEDIUM PRIORITY: Update Session Tests (Not Found / Authorization / Validation)
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_update_session_all_fields(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Create a session
    let created = poker_session::do_create_session(&db, user.id, default_session_request())
        .await
        .expect("Failed to create session");

    // Update all fields
    let update_req = UpdatePokerSessionRequest {
        session_date: Some("2024-02-20".to_string()),
        duration_minutes: Some(240),
        buy_in_amount: Some(500.0),
        rebuy_amount: Some(200.0),
        cash_out_amount: Some(1000.0),
        notes: Some("Updated notes".to_string()),
    };

    let updated = poker_session::do_update_session(&db, created.id, user.id, update_req)
        .expect("Failed to update session");

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.duration_minutes, 240);
    assert_eq!(updated.notes, Some("Updated notes".to_string()));
    assert_eq!(updated.buy_in_amount, BigDecimal::from_f64(500.0).unwrap());
}

#[rstest]
#[tokio::test]
async fn test_update_session_partial(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Create a session with specific values
    let session_req = CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 120,
        buy_in_amount: 100.0,
        rebuy_amount: Some(50.0),
        cash_out_amount: 200.0,
        notes: Some("Original notes".to_string()),
    };
    let created = poker_session::do_create_session(&db, user.id, session_req)
        .await
        .expect("Failed to create session");

    // Update only some fields
    let update_req = UpdatePokerSessionRequest {
        session_date: None,
        duration_minutes: Some(180),
        buy_in_amount: None,
        rebuy_amount: None,
        cash_out_amount: None,
        notes: None, // Keep original notes
    };

    let updated = poker_session::do_update_session(&db, created.id, user.id, update_req)
        .expect("Failed to update session");

    // Duration should be updated
    assert_eq!(updated.duration_minutes, 180);
    // Other fields should remain unchanged
    assert_eq!(updated.buy_in_amount, BigDecimal::from_f64(100.0).unwrap());
    assert_eq!(updated.notes, Some("Original notes".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_update_session_not_found(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    let fake_session_id = Uuid::new_v4();
    let update_req = UpdatePokerSessionRequest {
        session_date: None,
        duration_minutes: Some(180),
        buy_in_amount: None,
        rebuy_amount: None,
        cash_out_amount: None,
        notes: None,
    };

    let result = poker_session::do_update_session(&db, fake_session_id, user.id, update_req);

    assert!(matches!(result, Err(UpdateSessionError::NotFound)));
}

#[rstest]
#[tokio::test]
async fn test_update_session_wrong_user(#[future] test_db: TestDb) {
    let db = test_db.await;

    let user_a = create_test_user_raw(&db, "usera@test.com", "usera");
    let user_b = create_test_user_raw(&db, "userb@test.com", "userb");

    // Create a session for user A
    let session = poker_session::do_create_session(&db, user_a.id, default_session_request())
        .await
        .expect("Failed to create session");

    // User B tries to update user A's session
    let update_req = UpdatePokerSessionRequest {
        session_date: None,
        duration_minutes: Some(999),
        buy_in_amount: None,
        rebuy_amount: None,
        cash_out_amount: None,
        notes: None,
    };

    let result = poker_session::do_update_session(&db, session.id, user_b.id, update_req);

    assert!(matches!(result, Err(UpdateSessionError::NotFound)));

    // Verify session was not modified
    let original = poker_session::do_get_session(&db, session.id, user_a.id)
        .expect("Session should still exist");
    assert_eq!(original.duration_minutes, 120); // Original value
}

#[rstest]
#[tokio::test]
async fn test_update_session_invalid_date(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    let session = poker_session::do_create_session(&db, user.id, default_session_request())
        .await
        .expect("Failed to create session");

    // Try to update with invalid date format
    let update_req = UpdatePokerSessionRequest {
        session_date: Some("invalid-date".to_string()),
        duration_minutes: None,
        buy_in_amount: None,
        rebuy_amount: None,
        cash_out_amount: None,
        notes: None,
    };

    let result = poker_session::do_update_session(&db, session.id, user.id, update_req);

    assert!(matches!(result, Err(UpdateSessionError::InvalidDateFormat)));
}

// =============================================================================
// MEDIUM PRIORITY: Delete Session Tests (Not Found / Authorization)
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_delete_session_success(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Create a session
    let session = poker_session::do_create_session(&db, user.id, default_session_request())
        .await
        .expect("Failed to create session");

    // Delete the session
    poker_session::do_delete_session(&db, session.id, user.id).expect("Failed to delete session");

    // Verify session is gone
    let result = poker_session::do_get_session(&db, session.id, user.id);
    assert!(matches!(result, Err(GetSessionError::NotFound)));
}

#[rstest]
#[tokio::test]
async fn test_delete_session_not_found(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    let fake_session_id = Uuid::new_v4();
    let result = poker_session::do_delete_session(&db, fake_session_id, user.id);

    assert!(matches!(result, Err(DeleteSessionError::NotFound)));
}

#[rstest]
#[tokio::test]
async fn test_delete_session_wrong_user(#[future] test_db: TestDb) {
    let db = test_db.await;

    let user_a = create_test_user_raw(&db, "usera@test.com", "usera");
    let user_b = create_test_user_raw(&db, "userb@test.com", "userb");

    // Create a session for user A
    let session = poker_session::do_create_session(&db, user_a.id, default_session_request())
        .await
        .expect("Failed to create session");

    // User B tries to delete user A's session
    let result = poker_session::do_delete_session(&db, session.id, user_b.id);

    assert!(matches!(result, Err(DeleteSessionError::NotFound)));

    // Verify session still exists for user A
    let still_exists = poker_session::do_get_session(&db, session.id, user_a.id);
    assert!(still_exists.is_ok());
}

#[rstest]
#[tokio::test]
async fn test_delete_session_idempotent(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Create and delete a session
    let session = poker_session::do_create_session(&db, user.id, default_session_request())
        .await
        .expect("Failed to create session");

    poker_session::do_delete_session(&db, session.id, user.id)
        .expect("First delete should succeed");

    // Second delete should return NotFound
    let result = poker_session::do_delete_session(&db, session.id, user.id);
    assert!(matches!(result, Err(DeleteSessionError::NotFound)));
}

// =============================================================================
// MEDIUM PRIORITY: Validation Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_create_session_invalid_date_various_formats(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    let invalid_dates = vec![
        "2024/01/15",   // Wrong separator
        "15-01-2024",   // Day-Month-Year
        "Jan 15, 2024", // Month name
        "2024-13-01",   // Invalid month
        "2024-01-32",   // Invalid day
        "not-a-date",   // Complete garbage
        "",             // Empty string
    ];

    for invalid_date in invalid_dates {
        let session_req = CreatePokerSessionRequest {
            session_date: invalid_date.to_string(),
            duration_minutes: 120,
            buy_in_amount: 100.0,
            rebuy_amount: None,
            cash_out_amount: 150.0,
            notes: None,
        };

        let result = poker_session::do_create_session(&db, user.id, session_req).await;
        assert!(
            matches!(result, Err(CreateSessionError::InvalidDateFormat(_))),
            "Expected InvalidDateFormat for date: {}",
            invalid_date
        );
    }
}

#[rstest]
#[tokio::test]
async fn test_create_session_valid_date_formats(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Only YYYY-MM-DD format should work
    let session_req = CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 120,
        buy_in_amount: 100.0,
        rebuy_amount: None,
        cash_out_amount: 150.0,
        notes: None,
    };

    let result = poker_session::do_create_session(&db, user.id, session_req).await;
    assert!(result.is_ok());
}

#[rstest]
#[tokio::test]
async fn test_create_session_boundary_dates(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Test boundary dates
    let boundary_dates = vec![
        "2024-01-01", // Start of year
        "2024-12-31", // End of year
        "2024-02-29", // Leap year date
        "2000-01-01", // Y2K
        "1999-12-31", // Pre-Y2K
    ];

    for date in boundary_dates {
        let session_req = CreatePokerSessionRequest {
            session_date: date.to_string(),
            duration_minutes: 60,
            buy_in_amount: 100.0,
            rebuy_amount: None,
            cash_out_amount: 100.0,
            notes: None,
        };

        let result = poker_session::do_create_session(&db, user.id, session_req).await;
        assert!(result.is_ok(), "Date {} should be valid", date);
    }
}

#[rstest]
#[tokio::test]
async fn test_update_preserves_unmodified_fields(#[future] test_db: TestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "testuser");

    // Create with specific values
    let session_req = CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 120,
        buy_in_amount: 100.0,
        rebuy_amount: Some(50.0),
        cash_out_amount: 200.0,
        notes: Some("Original notes".to_string()),
    };
    let created = poker_session::do_create_session(&db, user.id, session_req)
        .await
        .expect("Failed to create session");

    // Update with empty request (all None)
    let update_req = UpdatePokerSessionRequest {
        session_date: None,
        duration_minutes: None,
        buy_in_amount: None,
        rebuy_amount: None,
        cash_out_amount: None,
        notes: None,
    };

    let updated = poker_session::do_update_session(&db, created.id, user.id, update_req)
        .expect("Failed to update session");

    // All original values should be preserved
    assert_eq!(updated.duration_minutes, 120);
    assert_eq!(updated.buy_in_amount, BigDecimal::from_f64(100.0).unwrap());
    assert_eq!(updated.rebuy_amount, BigDecimal::from_f64(50.0).unwrap());
    assert_eq!(
        updated.cash_out_amount,
        BigDecimal::from_f64(200.0).unwrap()
    );
    assert_eq!(updated.notes, Some("Original notes".to_string()));
}
