use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::poker_sessions;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct PokerSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_date: NaiveDate,
    pub duration_minutes: i32,
    pub buy_in_amount: BigDecimal,
    pub rebuy_amount: BigDecimal,
    pub cash_out_amount: BigDecimal,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate, Insertable)]
#[diesel(table_name = poker_sessions)]
pub struct NewPokerSession {
    pub user_id: Uuid,
    pub session_date: NaiveDate,
    #[validate(range(min = 1, message = "Duration must be at least 1 minute"))]
    pub duration_minutes: i32,
    pub buy_in_amount: BigDecimal,
    pub rebuy_amount: BigDecimal,
    pub cash_out_amount: BigDecimal,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePokerSessionRequest {
    pub session_date: String, // Will be parsed to NaiveDate
    #[validate(range(min = 1, message = "Duration must be at least 1 minute"))]
    pub duration_minutes: i32,
    pub buy_in_amount: f64,
    pub rebuy_amount: Option<f64>,
    pub cash_out_amount: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePokerSessionRequest {
    pub session_date: Option<String>,
    pub duration_minutes: Option<i32>,
    pub buy_in_amount: Option<f64>,
    pub rebuy_amount: Option<f64>,
    pub cash_out_amount: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionWithProfit {
    #[serde(flatten)]
    pub session: PokerSession,
    pub profit: f64,
}
