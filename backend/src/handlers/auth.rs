use axum::{
    Extension,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::models::{
    AuthResponse, ChangePasswordRequest, LoginRequest, NewUser, RegisterRequest,
    UpdateCookieConsent, User,
};
use crate::schema::users;
use crate::utils::create_jwt;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Response {
    if let Err(errors) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation failed",
                "details": errors.to_string()
            })),
        )
            .into_response();
    }

    let password_hash = match hash(&req.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to hash password"
                })),
            )
                .into_response();
        }
    };

    let new_user = NewUser {
        email: req.email.clone(),
        username: req.username.clone(),
        password_hash,
    };

    let mut conn = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Database connection failed"
                })),
            )
                .into_response();
        }
    };

    let user = match diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(&mut conn)
    {
        Ok(u) => u,
        Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            info,
        )) => {
            let message = info.message();
            let error_msg = if message.contains("email") {
                "An account with this email already exists"
            } else if message.contains("username") {
                "This username is already taken"
            } else {
                "An account with these details already exists"
            };
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "error": error_msg
                })),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to create account. Please try again."
                })),
            )
                .into_response();
        }
    };

    let token = match create_jwt(user.id) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Token generation failed"
                })),
            )
                .into_response();
        }
    };

    (StatusCode::CREATED, Json(AuthResponse { token, user })).into_response()
}

pub async fn login(State(state): State<Arc<AppState>>, Json(req): Json<LoginRequest>) -> Response {
    if let Err(errors) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation failed",
                "details": errors.to_string()
            })),
        )
            .into_response();
    }

    let mut conn = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Database connection failed"
                })),
            )
                .into_response();
        }
    };

    let user = match users::table
        .filter(users::email.eq(&req.email))
        .first::<User>(&mut conn)
    {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Invalid credentials"
                })),
            )
                .into_response();
        }
    };

    if !verify(&req.password, &user.password_hash).unwrap_or(false) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Invalid credentials"
            })),
        )
            .into_response();
    }

    let token = match create_jwt(user.id) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Token generation failed"
                })),
            )
                .into_response();
        }
    };

    (StatusCode::OK, Json(AuthResponse { token, user })).into_response()
}

pub async fn get_me(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Response {
    let mut conn = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Database connection failed"
                })),
            )
                .into_response();
        }
    };

    match users::table.find(user_id).first::<User>(&mut conn) {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "User not found"
            })),
        )
            .into_response(),
    }
}

pub async fn update_cookie_consent(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(consent): Json<UpdateCookieConsent>,
) -> Response {
    let mut conn = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Database connection failed"
                })),
            )
                .into_response();
        }
    };

    let consent_date = if consent.cookie_consent {
        Some(Utc::now().naive_utc())
    } else {
        None
    };

    match diesel::update(users::table.find(user_id))
        .set((
            users::cookie_consent.eq(consent.cookie_consent),
            users::cookie_consent_date.eq(consent_date),
            users::updated_at.eq(Utc::now().naive_utc()),
        ))
        .get_result::<User>(&mut conn)
    {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to update cookie consent"
            })),
        )
            .into_response(),
    }
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(passwords): Json<ChangePasswordRequest>,
) -> Response {
    if let Err(errors) = passwords.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation failed",
                "details": errors.to_string()
            })),
        )
            .into_response();
    }

    let mut conn = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Database connection failed"
                })),
            )
                .into_response();
        }
    };

    let user = match users::table.find(user_id).first::<User>(&mut conn) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "User not found"
                })),
            )
                .into_response();
        }
    };

    if !verify(&passwords.old_password, &user.password_hash).unwrap_or(false) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Current password is incorrect"
            })),
        )
            .into_response();
    }

    let new_password_hash = match hash(&passwords.new_password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to hash password"
                })),
            )
                .into_response();
        }
    };

    match diesel::update(users::table.find(user_id))
        .set((
            users::password_hash.eq(new_password_hash),
            users::updated_at.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Password changed successfully"
            })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to change password"
            })),
        )
            .into_response(),
    }
}
