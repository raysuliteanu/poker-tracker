use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: usize,  // expiration time
    pub iat: usize,  // issued at
}

pub fn create_jwt(user_id: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(7))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration as usize,
        iat: Utc::now().timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn decode_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            // SAFETY: Tests run single-threaded with --test-threads=1 or we use Once
            // to ensure this is only called once
            unsafe {
                env::set_var("JWT_SECRET", "test_secret_key_for_unit_tests");
            }
        });
    }

    #[test]
    fn test_create_jwt_returns_token() {
        setup();
        let user_id = Uuid::new_v4();
        let token = create_jwt(user_id);
        assert!(token.is_ok());
        assert!(!token.unwrap().is_empty());
    }

    #[test]
    fn test_create_and_decode_jwt_roundtrip() {
        setup();
        let user_id = Uuid::new_v4();
        let token = create_jwt(user_id).expect("should create token");
        let claims = decode_jwt(&token).expect("should decode token");
        assert_eq!(claims.sub, user_id.to_string());
    }

    #[test]
    fn test_decode_jwt_invalid_token() {
        setup();
        let result = decode_jwt("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_jwt_wrong_secret() {
        setup();
        let user_id = Uuid::new_v4();
        let token = create_jwt(user_id).expect("should create token");

        // Tamper with the token signature
        let mut parts: Vec<&str> = token.split('.').collect();
        if parts.len() == 3 {
            parts[2] = "invalid_signature";
        }
        let tampered_token = parts.join(".");

        let result = decode_jwt(&tampered_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_expiration_is_in_future() {
        setup();
        let user_id = Uuid::new_v4();
        let token = create_jwt(user_id).expect("should create token");
        let claims = decode_jwt(&token).expect("should decode token");

        let now = Utc::now().timestamp() as usize;
        assert!(claims.exp > now);
        // Should expire in ~7 days (allow some margin)
        let seven_days_from_now = now + (7 * 24 * 60 * 60);
        assert!(claims.exp <= seven_days_from_now + 60); // 60 second margin
    }

    #[test]
    fn test_claims_issued_at_is_recent() {
        setup();
        let user_id = Uuid::new_v4();
        let before = Utc::now().timestamp() as usize;
        let token = create_jwt(user_id).expect("should create token");
        let after = Utc::now().timestamp() as usize;
        let claims = decode_jwt(&token).expect("should decode token");

        assert!(claims.iat >= before);
        assert!(claims.iat <= after + 1); // 1 second margin
    }
}
