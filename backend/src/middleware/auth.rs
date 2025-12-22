use axum::{
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use std::task::{Context, Poll};
use thiserror::Error;
use tower::{Layer, Service};
use uuid::Uuid;

use crate::utils::jwt::decode_jwt;

/// Error type for token extraction failures
#[derive(Debug, Error, PartialEq)]
pub enum TokenError {
    #[error("Authorization header is missing")]
    Missing,
    #[error("Invalid authorization header format (expected 'Bearer <token>')")]
    InvalidFormat,
    #[error("Invalid or expired JWT token")]
    InvalidToken,
    #[error("Invalid user ID in token claims")]
    InvalidUserId,
}

/// Extract and validate a Bearer token from an Authorization header value.
/// Returns the user UUID if valid, or an error describing what went wrong.
pub fn extract_user_id_from_auth_header(
    auth_header: Option<&str>,
    jwt_secret: &str,
) -> Result<Uuid, TokenError> {
    let header = auth_header.ok_or(TokenError::Missing)?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or(TokenError::InvalidFormat)?;

    let claims = decode_jwt(token, jwt_secret).map_err(|_| TokenError::InvalidToken)?;

    Uuid::parse_str(&claims.sub).map_err(|_| TokenError::InvalidUserId)
}

/// Auth middleware as an Axum layer
#[derive(Clone)]
pub struct AuthLayer {
    jwt_secret: String,
}

impl AuthLayer {
    pub fn new(jwt_secret: String) -> Self {
        AuthLayer { jwt_secret }
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
            jwt_secret: self.jwt_secret.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthService<S> {
    inner: S,
    jwt_secret: String,
}

impl<S> Service<Request> for AuthService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        // Skip auth for public routes
        let path = req.uri().path();
        if path == "/api/health" || path == "/api/auth/register" || path == "/api/auth/login" {
            let future = self.inner.call(req);
            return Box::pin(future);
        }

        // Extract auth header
        let auth_header = req
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok());

        match extract_user_id_from_auth_header(auth_header, &self.jwt_secret) {
            Ok(user_id) => {
                // Insert user_id into request extensions
                let (mut parts, body) = req.into_parts();
                parts.extensions.insert(user_id);
                let req = Request::from_parts(parts, body);

                let future = self.inner.call(req);
                Box::pin(future)
            }
            Err(_) => {
                // Return unauthorized response
                Box::pin(async move {
                    Ok((
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"error": "Invalid or missing token"})),
                    )
                        .into_response())
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::jwt::create_jwt;
    use proptest::prelude::*;

    const TEST_SECRET: &str = "test_secret_key_for_testing";

    #[test]
    fn test_extract_user_id_missing_header() {
        let result = extract_user_id_from_auth_header(None, TEST_SECRET);
        assert_eq!(result, Err(TokenError::Missing));
    }

    #[test]
    fn test_extract_user_id_invalid_format() {
        let result = extract_user_id_from_auth_header(Some("InvalidFormat"), TEST_SECRET);
        assert_eq!(result, Err(TokenError::InvalidFormat));
    }

    #[test]
    fn test_extract_user_id_invalid_token() {
        let result = extract_user_id_from_auth_header(Some("Bearer invalid_token"), TEST_SECRET);
        assert_eq!(result, Err(TokenError::InvalidToken));
    }

    #[test]
    fn test_extract_user_id_success() {
        let user_id = Uuid::new_v4();
        let token = create_jwt(user_id, TEST_SECRET).unwrap();
        let auth_header = format!("Bearer {}", token);
        let result = extract_user_id_from_auth_header(Some(&auth_header), TEST_SECRET);
        assert_eq!(result, Ok(user_id));
    }

    #[test]
    fn test_extract_user_id_case_sensitive_bearer() {
        let user_id = Uuid::new_v4();
        let token = create_jwt(user_id, TEST_SECRET).unwrap();

        // Test lowercase "bearer" - should fail
        let auth_header = format!("bearer {}", token);
        let result = extract_user_id_from_auth_header(Some(&auth_header), TEST_SECRET);
        assert_eq!(result, Err(TokenError::InvalidFormat));
    }

    #[test]
    fn test_extract_user_id_with_whitespace() {
        let user_id = Uuid::new_v4();
        let token = create_jwt(user_id, TEST_SECRET).unwrap();

        // Test with extra whitespace
        let auth_header = format!("Bearer  {}", token);
        let result = extract_user_id_from_auth_header(Some(&auth_header), TEST_SECRET);
        // This should fail because strip_prefix expects exactly one space
        assert_eq!(result, Err(TokenError::InvalidToken));
    }

    #[test]
    fn test_extract_user_id_with_tampered_token() {
        let user_id = Uuid::new_v4();
        let mut token = create_jwt(user_id, TEST_SECRET).unwrap();

        // Tamper with the token by appending a character
        token.push('x');

        let auth_header = format!("Bearer {}", token);
        let result = extract_user_id_from_auth_header(Some(&auth_header), TEST_SECRET);
        assert_eq!(result, Err(TokenError::InvalidToken));
    }

    // Property-based tests for auth header parsing
    proptest! {
        #[test]
        fn missing_bearer_prefix_fails(s in "[a-zA-Z0-9_.-]{10,100}") {
            // Any string without "Bearer " prefix should fail
            if !s.starts_with("Bearer ") {
                let result = extract_user_id_from_auth_header(Some(&s), TEST_SECRET);
                prop_assert_eq!(result, Err(TokenError::InvalidFormat));
            }
        }

        #[test]
        fn lowercase_bearer_fails(token in "[a-zA-Z0-9_.-]{20,100}") {
            let auth_header = format!("bearer {}", token);
            let result = extract_user_id_from_auth_header(Some(&auth_header), TEST_SECRET);
            prop_assert_eq!(result, Err(TokenError::InvalidFormat));
        }

        #[test]
        fn uppercase_bearer_fails(token in "[a-zA-Z0-9_.-]{20,100}") {
            let auth_header = format!("BEARER {}", token);
            let result = extract_user_id_from_auth_header(Some(&auth_header), TEST_SECRET);
            prop_assert_eq!(result, Err(TokenError::InvalidFormat));
        }

        #[test]
        fn invalid_token_after_bearer_fails(token in "[a-zA-Z0-9]{10,50}") {
            // Random alphanumeric strings are not valid JWTs
            let auth_header = format!("Bearer {}", token);
            let result = extract_user_id_from_auth_header(Some(&auth_header), TEST_SECRET);
            prop_assert_eq!(result, Err(TokenError::InvalidToken));
        }

        #[test]
        fn valid_jwt_roundtrip_works(_dummy in 0..100_i32) {
            let user_id = Uuid::new_v4();
            let token = create_jwt(user_id, TEST_SECRET).unwrap();
            let auth_header = format!("Bearer {}", token);
            let result = extract_user_id_from_auth_header(Some(&auth_header), TEST_SECRET);
            prop_assert_eq!(result, Ok(user_id));
        }

        #[test]
        fn extra_spaces_after_bearer_fails(spaces in 2..=5_usize) {
            let user_id = Uuid::new_v4();
            let token = create_jwt(user_id, TEST_SECRET).unwrap();
            let space_str: String = (0..spaces).map(|_| ' ').collect();
            let auth_header = format!("Bearer{}{}", space_str, token);
            let result = extract_user_id_from_auth_header(Some(&auth_header), TEST_SECRET);
            // "Bearer  token" doesn't match "Bearer " prefix correctly
            prop_assert!(result.is_err());
        }

        #[test]
        fn token_with_prefix_whitespace_fails(spaces in 1..=3_usize) {
            let user_id = Uuid::new_v4();
            let token = create_jwt(user_id, TEST_SECRET).unwrap();
            let space_str: String = (0..spaces).map(|_| ' ').collect();
            let auth_header = format!("Bearer {}{}", space_str, token);
            // Leading whitespace in token part should cause invalid token
            let result = extract_user_id_from_auth_header(Some(&auth_header), TEST_SECRET);
            prop_assert_eq!(result, Err(TokenError::InvalidToken));
        }

        #[test]
        fn tampered_token_fails(char_to_append in "[a-zA-Z0-9]") {
            let user_id = Uuid::new_v4();
            let mut token = create_jwt(user_id, TEST_SECRET).unwrap();
            token.push_str(&char_to_append);
            let auth_header = format!("Bearer {}", token);
            let result = extract_user_id_from_auth_header(Some(&auth_header), TEST_SECRET);
            prop_assert_eq!(result, Err(TokenError::InvalidToken));
        }

        #[test]
        fn truncated_token_fails(truncate_amount in 1..=10_usize) {
            let user_id = Uuid::new_v4();
            let token = create_jwt(user_id, TEST_SECRET).unwrap();
            let truncated = if token.len() > truncate_amount {
                &token[..token.len() - truncate_amount]
            } else {
                ""
            };
            let auth_header = format!("Bearer {}", truncated);
            let result = extract_user_id_from_auth_header(Some(&auth_header), TEST_SECRET);
            prop_assert_eq!(result, Err(TokenError::InvalidToken));
        }
    }
}
