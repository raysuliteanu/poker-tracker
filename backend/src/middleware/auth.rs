use axum::{
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use uuid::Uuid;

use crate::utils::jwt::decode_jwt;

/// Error type for token extraction failures
#[derive(Debug, PartialEq)]
pub enum TokenError {
    Missing,
    InvalidFormat,
    InvalidToken,
    InvalidUserId,
}

/// Extract and validate a Bearer token from an Authorization header value.
/// Returns the user UUID if valid, or an error describing what went wrong.
pub fn extract_user_id_from_auth_header(auth_header: Option<&str>) -> Result<Uuid, TokenError> {
    let header = auth_header.ok_or(TokenError::Missing)?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or(TokenError::InvalidFormat)?;

    let claims = decode_jwt(token).map_err(|_| TokenError::InvalidToken)?;

    Uuid::parse_str(&claims.sub).map_err(|_| TokenError::InvalidUserId)
}

/// Auth middleware as an Axum layer
#[derive(Clone)]
pub struct AuthLayer;

impl AuthLayer {
    pub fn new() -> Self {
        AuthLayer
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService { inner }
    }
}

#[derive(Clone)]
pub struct AuthService<S> {
    inner: S,
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

        match extract_user_id_from_auth_header(auth_header) {
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
    use std::env;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init() {
        INIT.call_once(|| {
            unsafe { env::set_var("JWT_SECRET", "test_secret_key_for_testing") };
        });
    }

    #[test]
    fn test_extract_user_id_missing_header() {
        init();
        let result = extract_user_id_from_auth_header(None);
        assert_eq!(result, Err(TokenError::Missing));
    }

    #[test]
    fn test_extract_user_id_invalid_format() {
        init();
        let result = extract_user_id_from_auth_header(Some("InvalidFormat"));
        assert_eq!(result, Err(TokenError::InvalidFormat));
    }

    #[test]
    fn test_extract_user_id_invalid_token() {
        init();
        let result = extract_user_id_from_auth_header(Some("Bearer invalid_token"));
        assert_eq!(result, Err(TokenError::InvalidToken));
    }

    #[test]
    fn test_extract_user_id_success() {
        init();
        let user_id = Uuid::new_v4();
        let token = create_jwt(user_id).unwrap();
        let auth_header = format!("Bearer {}", token);
        let result = extract_user_id_from_auth_header(Some(&auth_header));
        assert_eq!(result, Ok(user_id));
    }

    #[test]
    fn test_extract_user_id_case_sensitive_bearer() {
        init();
        let user_id = Uuid::new_v4();
        let token = create_jwt(user_id).unwrap();

        // Test lowercase "bearer" - should fail
        let auth_header = format!("bearer {}", token);
        let result = extract_user_id_from_auth_header(Some(&auth_header));
        assert_eq!(result, Err(TokenError::InvalidFormat));
    }

    #[test]
    fn test_extract_user_id_with_whitespace() {
        init();
        let user_id = Uuid::new_v4();
        let token = create_jwt(user_id).unwrap();

        // Test with extra whitespace
        let auth_header = format!("Bearer  {}", token);
        let result = extract_user_id_from_auth_header(Some(&auth_header));
        // This should fail because strip_prefix expects exactly one space
        assert_eq!(result, Err(TokenError::InvalidToken));
    }

    #[test]
    fn test_extract_user_id_with_tampered_token() {
        init();
        let user_id = Uuid::new_v4();
        let mut token = create_jwt(user_id).unwrap();

        // Tamper with the token by appending a character
        token.push('x');

        let auth_header = format!("Bearer {}", token);
        let result = extract_user_id_from_auth_header(Some(&auth_header));
        assert_eq!(result, Err(TokenError::InvalidToken));
    }
}
