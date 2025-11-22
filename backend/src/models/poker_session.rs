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

/// Calculate profit from buy-in, rebuy, and cash-out amounts
pub fn calculate_profit(buy_in: &BigDecimal, rebuy: &BigDecimal, cash_out: &BigDecimal) -> f64 {
    let total_invested = buy_in + rebuy;
    (cash_out - &total_invested)
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::FromPrimitive;
    use chrono::Datelike;
    use validator::Validate;

    // CreatePokerSessionRequest validation tests
    #[test]
    fn test_create_session_request_valid() {
        let req = CreatePokerSessionRequest {
            session_date: "2024-01-15".to_string(),
            duration_minutes: 120,
            buy_in_amount: 100.0,
            rebuy_amount: Some(50.0),
            cash_out_amount: 200.0,
            notes: Some("Good session".to_string()),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_create_session_request_duration_zero() {
        let req = CreatePokerSessionRequest {
            session_date: "2024-01-15".to_string(),
            duration_minutes: 0,
            buy_in_amount: 100.0,
            rebuy_amount: None,
            cash_out_amount: 150.0,
            notes: None,
        };
        let result = req.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("duration_minutes"));
    }

    #[test]
    fn test_create_session_request_duration_negative() {
        let req = CreatePokerSessionRequest {
            session_date: "2024-01-15".to_string(),
            duration_minutes: -10,
            buy_in_amount: 100.0,
            rebuy_amount: None,
            cash_out_amount: 150.0,
            notes: None,
        };
        let result = req.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("duration_minutes"));
    }

    #[test]
    fn test_create_session_request_duration_boundary_valid() {
        let req = CreatePokerSessionRequest {
            session_date: "2024-01-15".to_string(),
            duration_minutes: 1, // minimum valid
            buy_in_amount: 100.0,
            rebuy_amount: None,
            cash_out_amount: 150.0,
            notes: None,
        };
        assert!(req.validate().is_ok());
    }

    // NewPokerSession validation tests
    #[test]
    fn test_new_poker_session_valid() {
        let session = NewPokerSession {
            user_id: Uuid::new_v4(),
            session_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            duration_minutes: 120,
            buy_in_amount: BigDecimal::from_f64(100.0).unwrap(),
            rebuy_amount: BigDecimal::from_f64(0.0).unwrap(),
            cash_out_amount: BigDecimal::from_f64(150.0).unwrap(),
            notes: None,
        };
        assert!(session.validate().is_ok());
    }

    #[test]
    fn test_new_poker_session_duration_zero() {
        let session = NewPokerSession {
            user_id: Uuid::new_v4(),
            session_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            duration_minutes: 0,
            buy_in_amount: BigDecimal::from_f64(100.0).unwrap(),
            rebuy_amount: BigDecimal::from_f64(0.0).unwrap(),
            cash_out_amount: BigDecimal::from_f64(150.0).unwrap(),
            notes: None,
        };
        let result = session.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("duration_minutes"));
    }

    // Profit calculation tests
    #[test]
    fn test_calculate_profit_positive() {
        let buy_in = BigDecimal::from_f64(100.0).unwrap();
        let rebuy = BigDecimal::from_f64(50.0).unwrap();
        let cash_out = BigDecimal::from_f64(200.0).unwrap();
        let profit = calculate_profit(&buy_in, &rebuy, &cash_out);
        assert!((profit - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_profit_negative() {
        let buy_in = BigDecimal::from_f64(100.0).unwrap();
        let rebuy = BigDecimal::from_f64(50.0).unwrap();
        let cash_out = BigDecimal::from_f64(100.0).unwrap();
        let profit = calculate_profit(&buy_in, &rebuy, &cash_out);
        assert!((profit - (-50.0)).abs() < 0.001);
    }

    #[test]
    fn test_calculate_profit_break_even() {
        let buy_in = BigDecimal::from_f64(100.0).unwrap();
        let rebuy = BigDecimal::from_f64(0.0).unwrap();
        let cash_out = BigDecimal::from_f64(100.0).unwrap();
        let profit = calculate_profit(&buy_in, &rebuy, &cash_out);
        assert!((profit - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_profit_no_rebuy() {
        let buy_in = BigDecimal::from_f64(200.0).unwrap();
        let rebuy = BigDecimal::from_f64(0.0).unwrap();
        let cash_out = BigDecimal::from_f64(500.0).unwrap();
        let profit = calculate_profit(&buy_in, &rebuy, &cash_out);
        assert!((profit - 300.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_profit_large_amounts() {
        let buy_in = BigDecimal::from_f64(10000.0).unwrap();
        let rebuy = BigDecimal::from_f64(5000.0).unwrap();
        let cash_out = BigDecimal::from_f64(25000.0).unwrap();
        let profit = calculate_profit(&buy_in, &rebuy, &cash_out);
        assert!((profit - 10000.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_profit_decimal_precision() {
        let buy_in = BigDecimal::from_f64(99.99).unwrap();
        let rebuy = BigDecimal::from_f64(50.01).unwrap();
        let cash_out = BigDecimal::from_f64(175.50).unwrap();
        let profit = calculate_profit(&buy_in, &rebuy, &cash_out);
        assert!((profit - 25.50).abs() < 0.01);
    }

    // Date parsing tests (testing the format used by handlers)
    #[test]
    fn test_date_parsing_valid() {
        let date_str = "2024-01-15";
        let result = NaiveDate::parse_from_str(date_str, "%Y-%m-%d");
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_date_parsing_invalid_format() {
        let date_str = "01/15/2024"; // wrong format
        let result = NaiveDate::parse_from_str(date_str, "%Y-%m-%d");
        assert!(result.is_err());
    }

    #[test]
    fn test_date_parsing_invalid_date() {
        let date_str = "2024-13-45"; // invalid month and day
        let result = NaiveDate::parse_from_str(date_str, "%Y-%m-%d");
        assert!(result.is_err());
    }
}
