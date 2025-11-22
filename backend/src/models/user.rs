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
    #[serde(skip_serializing)]
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

#[derive(Debug, Serialize)]
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
#[expect(dead_code)]
pub struct ResetPasswordRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
