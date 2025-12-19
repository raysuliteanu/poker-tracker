use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::users;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    #[serde(skip_serializing, default)]
    pub password_hash: String,
    pub cookie_consent: bool,
    pub cookie_consent_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(
        min = 3,
        max = 100,
        message = "Username must be between 3 and 100 characters"
    ))]
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(
        min = 3,
        max = 100,
        message = "Username must be between 3 and 100 characters"
    ))]
    pub username: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCookieConsent {
    pub cookie_consent: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
#[allow(dead_code)]
pub struct ResetPasswordRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use validator::Validate;

    // RegisterRequest validation tests
    #[test]
    fn test_register_request_valid() {
        let req = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "validuser".to_string(),
            password: "password123".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_register_request_invalid_email() {
        let req = RegisterRequest {
            email: "not-an-email".to_string(),
            username: "validuser".to_string(),
            password: "password123".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("email"));
    }

    #[test]
    fn test_register_request_username_too_short() {
        let req = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "ab".to_string(), // 2 chars, min is 3
            password: "password123".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("username"));
    }

    #[test]
    fn test_register_request_username_too_long() {
        let req = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "a".repeat(101), // 101 chars, max is 100
            password: "password123".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("username"));
    }

    #[test]
    fn test_register_request_username_boundary_valid() {
        // Test minimum boundary (3 chars)
        let req_min = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "abc".to_string(),
            password: "password123".to_string(),
        };
        assert!(req_min.validate().is_ok());

        // Test maximum boundary (100 chars)
        let req_max = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "a".repeat(100),
            password: "password123".to_string(),
        };
        assert!(req_max.validate().is_ok());
    }

    #[test]
    fn test_register_request_password_too_short() {
        let req = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "validuser".to_string(),
            password: "1234567".to_string(), // 7 chars, min is 8
        };
        let result = req.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("password"));
    }

    #[test]
    fn test_register_request_password_boundary_valid() {
        let req = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "validuser".to_string(),
            password: "12345678".to_string(), // exactly 8 chars
        };
        assert!(req.validate().is_ok());
    }

    // LoginRequest validation tests
    #[test]
    fn test_login_request_valid() {
        let req = LoginRequest {
            email: "test@example.com".to_string(),
            password: "anypassword".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_login_request_empty_password() {
        let req = LoginRequest {
            email: "test@example.com".to_string(),
            password: "".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("password"));
    }

    // ChangePasswordRequest validation tests
    #[test]
    fn test_change_password_request_valid() {
        let req = ChangePasswordRequest {
            old_password: "oldpassword".to_string(),
            new_password: "newpassword123".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_change_password_request_new_password_too_short() {
        let req = ChangePasswordRequest {
            old_password: "oldpassword".to_string(),
            new_password: "short".to_string(), // 5 chars, min is 8
        };
        let result = req.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("new_password"));
    }

    // NewUser validation tests
    #[test]
    fn test_new_user_valid() {
        let user = NewUser {
            email: "test@example.com".to_string(),
            username: "validuser".to_string(),
            password_hash: "hashed_password".to_string(),
        };
        assert!(user.validate().is_ok());
    }

    #[test]
    fn test_new_user_invalid_email() {
        let user = NewUser {
            email: "invalid-email".to_string(),
            username: "validuser".to_string(),
            password_hash: "hashed_password".to_string(),
        };
        let result = user.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("email"));
    }

    #[test]
    fn test_new_user_username_too_short() {
        let user = NewUser {
            email: "test@example.com".to_string(),
            username: "ab".to_string(),
            password_hash: "hashed_password".to_string(),
        };
        let result = user.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("username"));
    }

    // Property-based tests for username validation
    proptest! {
        #[test]
        fn valid_username_length_passes(len in 3..=100_usize) {
            let username: String = (0..len).map(|_| 'a').collect();
            let req = RegisterRequest {
                email: "test@example.com".to_string(),
                username,
                password: "password123".to_string(),
            };
            prop_assert!(req.validate().is_ok());
        }

        #[test]
        fn username_too_short_fails(len in 0..3_usize) {
            let username: String = (0..len).map(|_| 'a').collect();
            let req = RegisterRequest {
                email: "test@example.com".to_string(),
                username,
                password: "password123".to_string(),
            };
            let result = req.validate();
            prop_assert!(result.is_err());
            let errors = result.unwrap_err();
            prop_assert!(errors.field_errors().contains_key("username"));
        }

        #[test]
        fn username_too_long_fails(len in 101..=200_usize) {
            let username: String = (0..len).map(|_| 'a').collect();
            let req = RegisterRequest {
                email: "test@example.com".to_string(),
                username,
                password: "password123".to_string(),
            };
            let result = req.validate();
            prop_assert!(result.is_err());
            let errors = result.unwrap_err();
            prop_assert!(errors.field_errors().contains_key("username"));
        }

        #[test]
        fn username_with_various_valid_chars(
            prefix in "[a-zA-Z]{1,10}",
            middle in "[a-zA-Z0-9_-]{0,80}",
            suffix in "[a-zA-Z0-9]{1,9}",
        ) {
            let username = format!("{}{}{}", prefix, middle, suffix);
            if username.len() >= 3 && username.len() <= 100 {
                let req = RegisterRequest {
                    email: "test@example.com".to_string(),
                    username,
                    password: "password123".to_string(),
                };
                // Username length is valid, so validation should pass
                prop_assert!(req.validate().is_ok());
            }
        }
    }

    // Property-based tests for password validation
    proptest! {
        #[test]
        fn valid_password_length_passes(len in 8..=100_usize) {
            let password: String = (0..len).map(|_| 'x').collect();
            let req = RegisterRequest {
                email: "test@example.com".to_string(),
                username: "validuser".to_string(),
                password,
            };
            prop_assert!(req.validate().is_ok());
        }

        #[test]
        fn password_too_short_fails(len in 0..8_usize) {
            let password: String = (0..len).map(|_| 'x').collect();
            let req = RegisterRequest {
                email: "test@example.com".to_string(),
                username: "validuser".to_string(),
                password,
            };
            let result = req.validate();
            prop_assert!(result.is_err());
            let errors = result.unwrap_err();
            prop_assert!(errors.field_errors().contains_key("password"));
        }

        #[test]
        fn password_with_special_chars(
            base in "[a-zA-Z0-9]{4,8}",
            special in "[!@#$%^&*()]{2,4}",
        ) {
            let password = format!("{}{}", base, special);
            if password.len() >= 8 {
                let req = RegisterRequest {
                    email: "test@example.com".to_string(),
                    username: "validuser".to_string(),
                    password,
                };
                prop_assert!(req.validate().is_ok());
            }
        }
    }

    // Property-based tests for email validation
    proptest! {
        #[test]
        fn valid_email_format_passes(
            local in "[a-z]{1,10}",
            domain in "[a-z]{2,10}",
            tld in "(com|org|net|io)",
        ) {
            let email = format!("{}@{}.{}", local, domain, tld);
            let req = RegisterRequest {
                email,
                username: "validuser".to_string(),
                password: "password123".to_string(),
            };
            prop_assert!(req.validate().is_ok());
        }

        #[test]
        fn email_without_at_fails(s in "[a-z]{5,20}") {
            // String without @ should fail email validation
            let req = RegisterRequest {
                email: s,
                username: "validuser".to_string(),
                password: "password123".to_string(),
            };
            let result = req.validate();
            prop_assert!(result.is_err());
            let errors = result.unwrap_err();
            prop_assert!(errors.field_errors().contains_key("email"));
        }

        #[test]
        fn email_with_multiple_at_fails(
            local in "[a-z]{3,5}",
            middle in "[a-z]{3,5}",
            domain in "[a-z]{3,5}",
        ) {
            // email@with@multiple@at should fail
            let email = format!("{}@{}@{}.com", local, middle, domain);
            let req = RegisterRequest {
                email,
                username: "validuser".to_string(),
                password: "password123".to_string(),
            };
            let result = req.validate();
            prop_assert!(result.is_err());
        }
    }

    // Property-based tests for ChangePasswordRequest
    proptest! {
        #[test]
        fn change_password_valid_new_password(len in 8..=100_usize) {
            let new_password: String = (0..len).map(|_| 'y').collect();
            let req = ChangePasswordRequest {
                old_password: "oldpassword".to_string(),
                new_password,
            };
            prop_assert!(req.validate().is_ok());
        }

        #[test]
        fn change_password_invalid_new_password(len in 0..8_usize) {
            let new_password: String = (0..len).map(|_| 'y').collect();
            let req = ChangePasswordRequest {
                old_password: "oldpassword".to_string(),
                new_password,
            };
            let result = req.validate();
            prop_assert!(result.is_err());
            let errors = result.unwrap_err();
            prop_assert!(errors.field_errors().contains_key("new_password"));
        }
    }

    // Property-based tests for LoginRequest
    proptest! {
        #[test]
        fn login_with_non_empty_password_passes(len in 1..=100_usize) {
            let password: String = (0..len).map(|_| 'z').collect();
            let req = LoginRequest {
                email: "test@example.com".to_string(),
                password,
            };
            prop_assert!(req.validate().is_ok());
        }
    }
}
