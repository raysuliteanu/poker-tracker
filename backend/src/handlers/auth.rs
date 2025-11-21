use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    AuthResponse, ChangePasswordRequest, LoginRequest, NewUser, RegisterRequest,
    UpdateCookieConsent, User,
};
use crate::schema::users;
use crate::utils::{DbPool, create_jwt};

pub async fn register(pool: web::Data<DbPool>, req: web::Json<RegisterRequest>) -> impl Responder {
    if let Err(errors) = req.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    let password_hash = match hash(&req.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to hash password"
            }));
        }
    };

    let new_user = NewUser {
        email: req.email.clone(),
        username: req.username.clone(),
        password_hash,
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }));
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
            return HttpResponse::Conflict().json(serde_json::json!({
                "error": error_msg
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create account. Please try again."
            }));
        }
    };

    let token = match create_jwt(user.id) {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Token generation failed"
            }));
        }
    };

    HttpResponse::Created().json(AuthResponse { token, user })
}

pub async fn login(pool: web::Data<DbPool>, req: web::Json<LoginRequest>) -> impl Responder {
    if let Err(errors) = req.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }));
        }
    };

    let user = match users::table
        .filter(users::email.eq(&req.email))
        .first::<User>(&mut conn)
    {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid credentials"
            }));
        }
    };

    if !verify(&req.password, &user.password_hash).unwrap_or(false) {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid credentials"
        }));
    }

    let token = match create_jwt(user.id) {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Token generation failed"
            }));
        }
    };

    HttpResponse::Ok().json(AuthResponse { token, user })
}

pub async fn get_me(pool: web::Data<DbPool>, req: HttpRequest) -> impl Responder {
    let user_id = match req.extensions().get::<Uuid>() {
        Some(id) => *id,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized"
            }));
        }
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }));
        }
    };

    match users::table.find(user_id).first::<User>(&mut conn) {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        })),
    }
}

pub async fn update_cookie_consent(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    consent: web::Json<UpdateCookieConsent>,
) -> impl Responder {
    let user_id = match req.extensions().get::<Uuid>() {
        Some(id) => *id,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized"
            }));
        }
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }));
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
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to update cookie consent"
        })),
    }
}

pub async fn change_password(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    passwords: web::Json<ChangePasswordRequest>,
) -> impl Responder {
    if let Err(errors) = passwords.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    let user_id = match req.extensions().get::<Uuid>() {
        Some(id) => *id,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized"
            }));
        }
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }));
        }
    };

    let user = match users::table.find(user_id).first::<User>(&mut conn) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            }));
        }
    };

    if !verify(&passwords.old_password, &user.password_hash).unwrap_or(false) {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Current password is incorrect"
        }));
    }

    let new_password_hash = match hash(&passwords.new_password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to hash password"
            }));
        }
    };

    match diesel::update(users::table.find(user_id))
        .set((
            users::password_hash.eq(new_password_hash),
            users::updated_at.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Password changed successfully"
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to change password"
        })),
    }
}
