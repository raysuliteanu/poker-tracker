mod common;

use common::TestDb;
use diesel::{prelude::*, sql_types::Integer};
use poker_tracker::handlers::poker_session;
use poker_tracker::models::CreatePokerSessionRequest;
use poker_tracker::utils::DbConnectionProvider;
use rstest::rstest;

use crate::common::fixtures::test_db;

use poker_tracker::models::user::{NewUser, User};
use poker_tracker::schema::users;

#[rstest]
#[tokio::test]
async fn test_database_connection(#[future] test_db: TestDb) {
    let db = test_db.await;
    let mut conn = db.get_connection().expect("Failed to get db connection");
    let result = diesel::select(diesel::dsl::sql::<Integer>("1")).execute(&mut conn);
    assert!(result.is_ok());
}

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
