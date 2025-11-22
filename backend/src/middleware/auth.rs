use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures::future::LocalBoxFuture;
use std::future::{Ready, ready};
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

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok());

        if let Ok(user_id) = extract_user_id_from_auth_header(auth_header) {
            req.extensions_mut().insert(user_id);
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }

        Box::pin(async move {
            let response = HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid or missing token"}));
            Ok(req.into_response(response).map_into_right_body())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::jwt::create_jwt;
    use std::env;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            // SAFETY: Tests run single-threaded with Once guard
            unsafe {
                env::set_var("JWT_SECRET", "test_secret_key_for_unit_tests");
            }
        });
    }

    #[test]
    fn test_extract_user_id_missing_header() {
        setup();
        let result = extract_user_id_from_auth_header(None);
        assert_eq!(result, Err(TokenError::Missing));
    }

    #[test]
    fn test_extract_user_id_invalid_format_no_bearer() {
        setup();
        let result = extract_user_id_from_auth_header(Some("InvalidHeader"));
        assert_eq!(result, Err(TokenError::InvalidFormat));
    }

    #[test]
    fn test_extract_user_id_invalid_format_wrong_prefix() {
        setup();
        let result = extract_user_id_from_auth_header(Some("Basic sometoken"));
        assert_eq!(result, Err(TokenError::InvalidFormat));
    }

    #[test]
    fn test_extract_user_id_invalid_token() {
        setup();
        let result = extract_user_id_from_auth_header(Some("Bearer invalid.token.here"));
        assert_eq!(result, Err(TokenError::InvalidToken));
    }

    #[test]
    fn test_extract_user_id_valid_token() {
        setup();
        let user_id = Uuid::new_v4();
        let token = create_jwt(user_id).expect("should create token");
        let auth_header = format!("Bearer {}", token);

        let result = extract_user_id_from_auth_header(Some(&auth_header));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), user_id);
    }

    #[test]
    fn test_extract_user_id_empty_token() {
        setup();
        let result = extract_user_id_from_auth_header(Some("Bearer "));
        assert_eq!(result, Err(TokenError::InvalidToken));
    }

    #[test]
    fn test_extract_user_id_tampered_token() {
        setup();
        let user_id = Uuid::new_v4();
        let token = create_jwt(user_id).expect("should create token");

        // Tamper with the token
        let mut parts: Vec<&str> = token.split('.').collect();
        if parts.len() == 3 {
            parts[2] = "tampered_signature";
        }
        let tampered_token = parts.join(".");
        let auth_header = format!("Bearer {}", tampered_token);

        let result = extract_user_id_from_auth_header(Some(&auth_header));
        assert_eq!(result, Err(TokenError::InvalidToken));
    }

    #[test]
    fn test_extract_user_id_whitespace_handling() {
        setup();
        // Ensure extra whitespace is handled correctly
        let result = extract_user_id_from_auth_header(Some("Bearer  token_with_extra_space"));
        // The second space becomes part of the token, which is invalid
        assert_eq!(result, Err(TokenError::InvalidToken));
    }
}
