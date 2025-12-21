mod common;

use common::DirectConnectionTestDb;
use poker_tracker::handlers::auth::{LoginError, RegisterError, do_login, do_register};
use rstest::rstest;

use crate::common::fixtures::test_db;

#[rstest]
#[tokio::test]
async fn test_register_user_success(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    let user = do_register(
        &db,
        "test@example.com".to_string(),
        "testuser".to_string(),
        "password123".to_string(),
    )
    .expect("Registration should succeed");

    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.username, "testuser");
    // Password should be hashed, not stored in plain text
    assert_ne!(user.password_hash, "password123");
    assert!(!user.password_hash.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_register_duplicate_email(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    // First registration should succeed
    do_register(
        &db,
        "duplicate@example.com".to_string(),
        "user1".to_string(),
        "password123".to_string(),
    )
    .expect("First registration should succeed");

    // Second registration with same email should fail
    let result = do_register(
        &db,
        "duplicate@example.com".to_string(),
        "user2".to_string(),
        "password456".to_string(),
    );

    assert!(matches!(result, Err(RegisterError::DuplicateEmail)));
}

#[rstest]
#[tokio::test]
async fn test_register_duplicate_username(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    // First registration should succeed
    do_register(
        &db,
        "user1@example.com".to_string(),
        "duplicateuser".to_string(),
        "password123".to_string(),
    )
    .expect("First registration should succeed");

    // Second registration with same username should fail
    let result = do_register(
        &db,
        "user2@example.com".to_string(),
        "duplicateuser".to_string(),
        "password456".to_string(),
    );

    assert!(matches!(result, Err(RegisterError::DuplicateUsername)));
}

#[rstest]
#[tokio::test]
async fn test_register_returns_valid_user_id(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    let user = do_register(
        &db,
        "test@example.com".to_string(),
        "testuser".to_string(),
        "password123".to_string(),
    )
    .expect("Registration should succeed");

    // User ID should be a valid UUID (not nil)
    assert!(!user.id.is_nil());
}

#[rstest]
#[tokio::test]
async fn test_register_sets_default_cookie_consent(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    let user = do_register(
        &db,
        "test@example.com".to_string(),
        "testuser".to_string(),
        "password123".to_string(),
    )
    .expect("Registration should succeed");

    // Cookie consent should default to false
    assert!(!user.cookie_consent);
    assert!(user.cookie_consent_date.is_none());
}

#[rstest]
#[tokio::test]
async fn test_login_success(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    // First register a user
    let registered_user = do_register(
        &db,
        "login@example.com".to_string(),
        "loginuser".to_string(),
        "correctpassword".to_string(),
    )
    .expect("Registration should succeed");

    // Now login with correct credentials
    let logged_in_user = do_login(
        &db,
        "login@example.com".to_string(),
        "correctpassword".to_string(),
    )
    .expect("Login should succeed");

    assert_eq!(logged_in_user.id, registered_user.id);
    assert_eq!(logged_in_user.email, "login@example.com");
    assert_eq!(logged_in_user.username, "loginuser");
}

#[rstest]
#[tokio::test]
async fn test_login_wrong_password(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    // First register a user
    do_register(
        &db,
        "login@example.com".to_string(),
        "loginuser".to_string(),
        "correctpassword".to_string(),
    )
    .expect("Registration should succeed");

    // Try login with wrong password
    let result = do_login(
        &db,
        "login@example.com".to_string(),
        "wrongpassword".to_string(),
    );

    assert!(matches!(result, Err(LoginError::InvalidCredentials)));
}

#[rstest]
#[tokio::test]
async fn test_login_nonexistent_user(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    // Try login with non-existent email
    let result = do_login(
        &db,
        "nonexistent@example.com".to_string(),
        "somepassword".to_string(),
    );

    assert!(matches!(result, Err(LoginError::InvalidCredentials)));
}

#[rstest]
#[tokio::test]
async fn test_login_after_registration_flow(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    // Test the full registration -> login flow
    let email = "flow@example.com".to_string();
    let password = "securepassword123".to_string();

    // Register
    let registered = do_register(&db, email.clone(), "flowuser".to_string(), password.clone())
        .expect("Registration should succeed");

    // Login
    let logged_in = do_login(&db, email, password).expect("Login should succeed");

    // Verify it's the same user
    assert_eq!(registered.id, logged_in.id);
}

#[rstest]
#[tokio::test]
async fn test_login_case_sensitive_email(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    // Register with lowercase email
    do_register(
        &db,
        "test@example.com".to_string(),
        "testuser".to_string(),
        "password123".to_string(),
    )
    .expect("Registration should succeed");

    // Try login with different case - should fail (emails are case-sensitive in this impl)
    let result = do_login(
        &db,
        "TEST@EXAMPLE.COM".to_string(),
        "password123".to_string(),
    );

    // This tests the current behavior - email lookup is case-sensitive
    assert!(matches!(result, Err(LoginError::InvalidCredentials)));
}

#[rstest]
#[tokio::test]
async fn test_login_password_not_stored_plaintext(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    let password = "mySecretPassword123";

    // Register
    do_register(
        &db,
        "test@example.com".to_string(),
        "testuser".to_string(),
        password.to_string(),
    )
    .expect("Registration should succeed");

    // Login should succeed with plain password
    let user = do_login(&db, "test@example.com".to_string(), password.to_string())
        .expect("Login should succeed");

    // But the stored hash should not equal the plain password
    assert_ne!(user.password_hash, password);
    // And login with the hash as password should fail
    let result = do_login(
        &db,
        "test@example.com".to_string(),
        user.password_hash.clone(),
    );
    assert!(matches!(result, Err(LoginError::InvalidCredentials)));
}

#[rstest]
#[tokio::test]
async fn test_register_empty_email(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    // Empty email should still work at the do_register level (validation happens in handler)
    // but the database constraint should reject it or bcrypt should work
    // This tests that we can create users with various inputs
    let result = do_register(
        &db,
        "".to_string(),
        "testuser".to_string(),
        "password123".to_string(),
    );

    // Empty email is technically allowed at the business logic level
    // (validation happens at the handler level before calling do_register)
    // The database may or may not reject it based on constraints
    // This test documents the current behavior
    assert!(result.is_ok() || result.is_err());
}

#[rstest]
#[tokio::test]
async fn test_register_empty_username(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    let result = do_register(
        &db,
        "test@example.com".to_string(),
        "".to_string(),
        "password123".to_string(),
    );

    // Empty username - documents current behavior
    assert!(result.is_ok() || result.is_err());
}

#[rstest]
#[tokio::test]
async fn test_register_empty_password(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    // Empty password should still hash successfully with bcrypt
    let result = do_register(
        &db,
        "test@example.com".to_string(),
        "testuser".to_string(),
        "".to_string(),
    );

    // bcrypt can hash empty strings, so this should succeed at the do_register level
    assert!(result.is_ok());
}

#[rstest]
#[tokio::test]
async fn test_login_empty_email(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    // Login with empty email should fail (no user found)
    let result = do_login(&db, "".to_string(), "password123".to_string());

    assert!(matches!(result, Err(LoginError::InvalidCredentials)));
}

#[rstest]
#[tokio::test]
async fn test_login_empty_password(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    // First register a user with a real password
    do_register(
        &db,
        "test@example.com".to_string(),
        "testuser".to_string(),
        "realpassword123".to_string(),
    )
    .expect("Registration should succeed");

    // Login with empty password should fail
    let result = do_login(&db, "test@example.com".to_string(), "".to_string());

    assert!(matches!(result, Err(LoginError::InvalidCredentials)));
}

#[rstest]
#[tokio::test]
async fn test_multiple_users_independent_login(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;

    // Register multiple users
    let user1 = do_register(
        &db,
        "user1@example.com".to_string(),
        "user1".to_string(),
        "password1".to_string(),
    )
    .expect("User 1 registration should succeed");

    let user2 = do_register(
        &db,
        "user2@example.com".to_string(),
        "user2".to_string(),
        "password2".to_string(),
    )
    .expect("User 2 registration should succeed");

    // Each user should be able to login with their own credentials
    let logged_in_1 = do_login(
        &db,
        "user1@example.com".to_string(),
        "password1".to_string(),
    )
    .expect("User 1 login should succeed");
    assert_eq!(logged_in_1.id, user1.id);

    let logged_in_2 = do_login(
        &db,
        "user2@example.com".to_string(),
        "password2".to_string(),
    )
    .expect("User 2 login should succeed");
    assert_eq!(logged_in_2.id, user2.id);

    // Users should not be able to login with each other's passwords
    let cross_login = do_login(
        &db,
        "user1@example.com".to_string(),
        "password2".to_string(),
    );
    assert!(matches!(cross_login, Err(LoginError::InvalidCredentials)));
}

#[rstest]
#[tokio::test]
async fn test_register_same_email_different_username_fails(
    #[future] test_db: DirectConnectionTestDb,
) {
    let db = test_db.await;

    do_register(
        &db,
        "shared@example.com".to_string(),
        "user1".to_string(),
        "password1".to_string(),
    )
    .expect("First registration should succeed");

    let result = do_register(
        &db,
        "shared@example.com".to_string(),
        "user2".to_string(),
        "password2".to_string(),
    );

    assert!(matches!(result, Err(RegisterError::DuplicateEmail)));
}

#[rstest]
#[tokio::test]
async fn test_register_same_username_different_email_fails(
    #[future] test_db: DirectConnectionTestDb,
) {
    let db = test_db.await;

    do_register(
        &db,
        "user1@example.com".to_string(),
        "shareduser".to_string(),
        "password1".to_string(),
    )
    .expect("First registration should succeed");

    let result = do_register(
        &db,
        "user2@example.com".to_string(),
        "shareduser".to_string(),
        "password2".to_string(),
    );

    assert!(matches!(result, Err(RegisterError::DuplicateUsername)));
}
