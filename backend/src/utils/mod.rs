pub mod db;
pub mod jwt;

pub use db::*;
pub use jwt::*;

/// Get the bcrypt cost from environment variable or use default.
/// Allows configuring hashing cost for different environments:
/// - Production: BCRYPT_COST=12 (secure, default)
/// - Load testing: BCRYPT_COST=4 (fast)
pub fn get_bcrypt_cost() -> u32 {
    std::env::var("BCRYPT_COST")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(bcrypt::DEFAULT_COST)
}
