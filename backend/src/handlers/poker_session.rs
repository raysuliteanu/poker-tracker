use axum::{
    Extension,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Json, Response},
};
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::models::{
    CreatePokerSessionRequest, NewPokerSession, PokerSession, SessionWithProfit,
    UpdatePokerSessionRequest, calculate_profit,
};
use crate::schema::poker_sessions;
use crate::utils::DbProvider;

#[derive(Debug, Error)]
pub enum CreateSessionError {
    #[error("Invalid date format: {0}")]
    InvalidDateFormat(String),
    #[error("Database connection error: {0}")]
    DatabaseConnection(String),
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),
}

#[derive(Debug, Error)]
pub enum GetSessionError {
    #[error("Database connection error")]
    DatabaseConnection,
    #[error("Session not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum UpdateSessionError {
    #[error("Database connection error")]
    DatabaseConnection,
    #[error("Session not found")]
    NotFound,
    #[error("Invalid date format")]
    InvalidDateFormat,
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),
}

#[derive(Debug, Error)]
pub enum DeleteSessionError {
    #[error("Database connection error")]
    DatabaseConnection,
    #[error("Session not found")]
    NotFound,
}

pub async fn do_create_session(
    db_provider: &dyn DbProvider,
    user_id: Uuid,
    session_req: CreatePokerSessionRequest,
) -> Result<PokerSession, CreateSessionError> {
    let session_date = NaiveDate::parse_from_str(&session_req.session_date, "%Y-%m-%d")
        .map_err(|e| CreateSessionError::InvalidDateFormat(e.to_string()))?;

    let new_session = NewPokerSession {
        user_id,
        session_date,
        duration_minutes: session_req.duration_minutes,
        buy_in_amount: BigDecimal::from_f64(session_req.buy_in_amount).unwrap(),
        rebuy_amount: BigDecimal::from_f64(session_req.rebuy_amount.unwrap_or(0.0)).unwrap(),
        cash_out_amount: BigDecimal::from_f64(session_req.cash_out_amount).unwrap(),
        notes: session_req.notes.clone(),
    };

    let mut conn = db_provider.get_connection().map_err(|_| {
        CreateSessionError::DatabaseConnection("Failed to get connection".to_string())
    })?;

    Ok(diesel::insert_into(poker_sessions::table)
        .values(&new_session)
        .get_result::<PokerSession>(&mut conn)?)
}

/// Business logic for getting a single session
pub fn do_get_session(
    db_provider: &dyn DbProvider,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<PokerSession, GetSessionError> {
    let mut conn = db_provider
        .get_connection()
        .map_err(|_| GetSessionError::DatabaseConnection)?;

    poker_sessions::table
        .filter(poker_sessions::id.eq(session_id))
        .filter(poker_sessions::user_id.eq(user_id))
        .first::<PokerSession>(&mut conn)
        .map_err(|_| GetSessionError::NotFound)
}

/// Business logic for updating a session
pub fn do_update_session(
    db_provider: &dyn DbProvider,
    session_id: Uuid,
    user_id: Uuid,
    update_req: UpdatePokerSessionRequest,
) -> Result<PokerSession, UpdateSessionError> {
    let mut conn = db_provider
        .get_connection()
        .map_err(|_| UpdateSessionError::DatabaseConnection)?;

    // First verify ownership and get existing session
    let existing_session = poker_sessions::table
        .filter(poker_sessions::id.eq(session_id))
        .filter(poker_sessions::user_id.eq(user_id))
        .first::<PokerSession>(&mut conn)
        .map_err(|_| UpdateSessionError::NotFound)?;

    // Parse date if provided
    let session_date = if let Some(date_str) = &update_req.session_date {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| UpdateSessionError::InvalidDateFormat)?
    } else {
        existing_session.session_date
    };

    let duration_minutes = update_req
        .duration_minutes
        .unwrap_or(existing_session.duration_minutes);

    let buy_in_amount = update_req
        .buy_in_amount
        .map(|v| BigDecimal::from_f64(v).unwrap())
        .unwrap_or(existing_session.buy_in_amount);

    let rebuy_amount = update_req
        .rebuy_amount
        .map(|v| BigDecimal::from_f64(v).unwrap())
        .unwrap_or(existing_session.rebuy_amount);

    let cash_out_amount = update_req
        .cash_out_amount
        .map(|v| BigDecimal::from_f64(v).unwrap())
        .unwrap_or(existing_session.cash_out_amount);

    let notes = update_req.notes.clone().or(existing_session.notes);

    diesel::update(poker_sessions::table.find(existing_session.id))
        .set((
            poker_sessions::session_date.eq(session_date),
            poker_sessions::duration_minutes.eq(duration_minutes),
            poker_sessions::buy_in_amount.eq(buy_in_amount),
            poker_sessions::rebuy_amount.eq(rebuy_amount),
            poker_sessions::cash_out_amount.eq(cash_out_amount),
            poker_sessions::notes.eq(notes),
            poker_sessions::updated_at.eq(Utc::now().naive_utc()),
        ))
        .get_result::<PokerSession>(&mut conn)
        .map_err(UpdateSessionError::Database)
}

/// Business logic for deleting a session
pub fn do_delete_session(
    db_provider: &dyn DbProvider,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<(), DeleteSessionError> {
    let mut conn = db_provider
        .get_connection()
        .map_err(|_| DeleteSessionError::DatabaseConnection)?;

    let count = diesel::delete(
        poker_sessions::table
            .filter(poker_sessions::id.eq(session_id))
            .filter(poker_sessions::user_id.eq(user_id)),
    )
    .execute(&mut conn)
    .map_err(|_| DeleteSessionError::NotFound)?;

    if count > 0 {
        Ok(())
    } else {
        Err(DeleteSessionError::NotFound)
    }
}

pub async fn create_session(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(session_req): Json<CreatePokerSessionRequest>,
) -> Response {
    if let Err(errors) = session_req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation failed",
                "details": errors.to_string()
            })),
        )
            .into_response();
    }

    match do_create_session(state.db_provider.as_ref(), user_id, session_req).await {
        Ok(session) => {
            let profit = calculate_profit(
                &session.buy_in_amount,
                &session.rebuy_amount,
                &session.cash_out_amount,
            );
            (
                StatusCode::CREATED,
                Json(SessionWithProfit { session, profit }),
            )
                .into_response()
        }
        Err(CreateSessionError::InvalidDateFormat(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Invalid date format: {}", msg)
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to create session: {}", e)
            })),
        )
            .into_response(),
    }
}

pub async fn get_sessions(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Response {
    let mut conn = match state.db_provider.get_connection() {
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

    match poker_sessions::table
        .filter(poker_sessions::user_id.eq(user_id))
        .order(poker_sessions::session_date.desc())
        .load::<PokerSession>(&mut conn)
    {
        Ok(sessions) => {
            let sessions_with_profit: Vec<SessionWithProfit> = sessions
                .into_iter()
                .map(|s| {
                    let profit =
                        calculate_profit(&s.buy_in_amount, &s.rebuy_amount, &s.cash_out_amount);
                    SessionWithProfit { session: s, profit }
                })
                .collect();
            (StatusCode::OK, Json(sessions_with_profit)).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to fetch sessions"
            })),
        )
            .into_response(),
    }
}

pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(session_id): Path<Uuid>,
) -> Response {
    match do_get_session(state.db_provider.as_ref(), session_id, user_id) {
        Ok(session) => {
            let profit = calculate_profit(
                &session.buy_in_amount,
                &session.rebuy_amount,
                &session.cash_out_amount,
            );
            (StatusCode::OK, Json(SessionWithProfit { session, profit })).into_response()
        }
        Err(GetSessionError::DatabaseConnection) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Database connection failed"
            })),
        )
            .into_response(),
        Err(GetSessionError::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Session not found"
            })),
        )
            .into_response(),
    }
}

pub async fn update_session(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(session_id): Path<Uuid>,
    Json(update_req): Json<UpdatePokerSessionRequest>,
) -> Response {
    match do_update_session(state.db_provider.as_ref(), session_id, user_id, update_req) {
        Ok(session) => {
            let profit = calculate_profit(
                &session.buy_in_amount,
                &session.rebuy_amount,
                &session.cash_out_amount,
            );
            (StatusCode::OK, Json(SessionWithProfit { session, profit })).into_response()
        }
        Err(UpdateSessionError::DatabaseConnection) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Database connection failed"
            })),
        )
            .into_response(),
        Err(UpdateSessionError::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Session not found"
            })),
        )
            .into_response(),
        Err(UpdateSessionError::InvalidDateFormat) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid date format. Expected YYYY-MM-DD"
            })),
        )
            .into_response(),
        Err(UpdateSessionError::Database(_)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to update session"
            })),
        )
            .into_response(),
    }
}

pub async fn delete_session(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(session_id): Path<Uuid>,
) -> Response {
    match do_delete_session(state.db_provider.as_ref(), session_id, user_id) {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Session deleted successfully"
            })),
        )
            .into_response(),
        Err(DeleteSessionError::DatabaseConnection) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Database connection failed"
            })),
        )
            .into_response(),
        Err(DeleteSessionError::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Session not found"
            })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub time_range: Option<String>,
}

pub async fn export_sessions(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Query(query): Query<ExportQuery>,
) -> Response {
    let mut conn = match state.db_provider.get_connection() {
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

    // Calculate cutoff date based on time range
    let cutoff_date = match query.time_range.as_deref() {
        Some("7days") => Some(Utc::now().naive_utc().date() - chrono::Duration::days(7)),
        Some("30days") => Some(Utc::now().naive_utc().date() - chrono::Duration::days(30)),
        Some("90days") => Some(Utc::now().naive_utc().date() - chrono::Duration::days(90)),
        Some("1year") => Some(Utc::now().naive_utc().date() - chrono::Duration::days(365)),
        Some("all") | None => None,
        Some(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid time_range. Valid options: 7days, 30days, 90days, 1year, all"
                })),
            )
                .into_response();
        }
    };

    // Query sessions with optional date filter
    let sessions: Vec<PokerSession> = match cutoff_date {
        Some(date) => poker_sessions::table
            .filter(poker_sessions::user_id.eq(user_id))
            .filter(poker_sessions::session_date.ge(date))
            .order(poker_sessions::session_date.asc())
            .load::<PokerSession>(&mut conn),
        None => poker_sessions::table
            .filter(poker_sessions::user_id.eq(user_id))
            .order(poker_sessions::session_date.asc())
            .load::<PokerSession>(&mut conn),
    }
    .unwrap_or_else(|_| vec![]);

    // Generate CSV
    let csv = generate_csv(&sessions);

    let filename = format!(
        "attachment; filename=\"poker-sessions-{}.csv\"",
        query.time_range.as_deref().unwrap_or("all")
    );

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (header::CONTENT_DISPOSITION, &filename),
        ],
        csv,
    )
        .into_response()
}

fn generate_csv(sessions: &[PokerSession]) -> String {
    let mut csv = String::from("Date,Duration (hours),Buy-in,Rebuy,Cash Out,Profit/Loss,Notes\n");

    for session in sessions {
        let profit = calculate_profit(
            &session.buy_in_amount,
            &session.rebuy_amount,
            &session.cash_out_amount,
        );
        let duration_hours = session.duration_minutes as f64 / 60.0;
        let notes = session.notes.as_deref().unwrap_or("");
        let escaped_notes = escape_csv_field(notes);

        csv.push_str(&format!(
            "{},{:.1},{},{},{},{:.2},{}\n",
            session.session_date,
            duration_hours,
            session.buy_in_amount,
            session.rebuy_amount,
            session.cash_out_amount,
            profit,
            escaped_notes
        ));
    }

    csv
}

fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::FromPrimitive;
    use chrono::{NaiveDate, Utc};
    use proptest::prelude::*;

    // CSV field escaping tests
    #[test]
    fn test_escape_csv_field_no_escaping_needed() {
        let field = "Simple text";
        let result = escape_csv_field(field);
        assert_eq!(result, "Simple text");
    }

    #[test]
    fn test_escape_csv_field_with_comma() {
        let field = "Text, with comma";
        let result = escape_csv_field(field);
        assert_eq!(result, "\"Text, with comma\"");
    }

    #[test]
    fn test_escape_csv_field_with_quotes() {
        let field = "Text with \"quotes\"";
        let result = escape_csv_field(field);
        assert_eq!(result, "\"Text with \"\"quotes\"\"\"");
    }

    #[test]
    fn test_escape_csv_field_with_newline() {
        let field = "Text with\nnewline";
        let result = escape_csv_field(field);
        assert_eq!(result, "\"Text with\nnewline\"");
    }

    #[test]
    fn test_escape_csv_field_empty() {
        let field = "";
        let result = escape_csv_field(field);
        assert_eq!(result, "");
    }

    #[test]
    fn test_escape_csv_field_multiple_special_chars() {
        let field = "Text, with \"quotes\" and\nnewlines";
        let result = escape_csv_field(field);
        assert_eq!(result, "\"Text, with \"\"quotes\"\" and\nnewlines\"");
    }

    // CSV generation tests
    #[test]
    fn test_generate_csv_empty() {
        let sessions: Vec<PokerSession> = vec![];
        let csv = generate_csv(&sessions);
        assert_eq!(
            csv,
            "Date,Duration (hours),Buy-in,Rebuy,Cash Out,Profit/Loss,Notes\n"
        );
    }

    #[test]
    fn test_generate_csv_single_session() {
        let session = PokerSession {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            session_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            duration_minutes: 120,
            buy_in_amount: BigDecimal::from_f64(100.0).unwrap(),
            rebuy_amount: BigDecimal::from_f64(50.0).unwrap(),
            cash_out_amount: BigDecimal::from_f64(200.0).unwrap(),
            notes: Some("Good session".to_string()),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

        let csv = generate_csv(&[session]);
        let lines: Vec<&str> = csv.lines().collect();

        assert_eq!(lines.len(), 2); // header + 1 data row
        assert_eq!(
            lines[0],
            "Date,Duration (hours),Buy-in,Rebuy,Cash Out,Profit/Loss,Notes"
        );
        assert!(lines[1].contains("2024-01-15"));
        assert!(lines[1].contains("2.0")); // 120 minutes = 2.0 hours
        assert!(lines[1].contains("100"));
        assert!(lines[1].contains("50"));
        assert!(lines[1].contains("200"));
        assert!(lines[1].contains("50.00")); // profit
        assert!(lines[1].contains("Good session"));
    }

    #[test]
    fn test_generate_csv_multiple_sessions() {
        let sessions = vec![
            PokerSession {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                session_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                duration_minutes: 120,
                buy_in_amount: BigDecimal::from_f64(100.0).unwrap(),
                rebuy_amount: BigDecimal::from_f64(0.0).unwrap(),
                cash_out_amount: BigDecimal::from_f64(150.0).unwrap(),
                notes: None,
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            },
            PokerSession {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                session_date: NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
                duration_minutes: 180,
                buy_in_amount: BigDecimal::from_f64(200.0).unwrap(),
                rebuy_amount: BigDecimal::from_f64(100.0).unwrap(),
                cash_out_amount: BigDecimal::from_f64(250.0).unwrap(),
                notes: Some("Lost session".to_string()),
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            },
        ];

        let csv = generate_csv(&sessions);
        let lines: Vec<&str> = csv.lines().collect();

        assert_eq!(lines.len(), 3); // header + 2 data rows
    }

    #[test]
    fn test_generate_csv_with_special_chars_in_notes() {
        let session = PokerSession {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            session_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            duration_minutes: 60,
            buy_in_amount: BigDecimal::from_f64(100.0).unwrap(),
            rebuy_amount: BigDecimal::from_f64(0.0).unwrap(),
            cash_out_amount: BigDecimal::from_f64(100.0).unwrap(),
            notes: Some("Notes with, comma and \"quotes\"".to_string()),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

        let csv = generate_csv(&[session]);
        let lines: Vec<&str> = csv.lines().collect();

        // The notes field should be escaped with quotes
        assert!(lines[1].contains("\"Notes with, comma and \"\"quotes\"\"\""));
    }

    #[test]
    fn test_generate_csv_negative_profit() {
        let session = PokerSession {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            session_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            duration_minutes: 90,
            buy_in_amount: BigDecimal::from_f64(200.0).unwrap(),
            rebuy_amount: BigDecimal::from_f64(100.0).unwrap(),
            cash_out_amount: BigDecimal::from_f64(200.0).unwrap(),
            notes: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

        let csv = generate_csv(&[session]);
        let lines: Vec<&str> = csv.lines().collect();

        // Should show -100.00 profit
        assert!(lines[1].contains("-100.00"));
    }

    #[test]
    fn test_generate_csv_duration_conversion() {
        // Test various duration conversions to hours
        let test_cases = vec![
            (60, "1.0"),  // 60 minutes = 1.0 hour
            (90, "1.5"),  // 90 minutes = 1.5 hours
            (120, "2.0"), // 120 minutes = 2.0 hours
            (45, "0.8"),  // 45 minutes = 0.75 hours (rounded to 0.8)
            (1, "0.0"),   // 1 minute = 0.0 hours (rounded)
        ];

        for (minutes, expected_hours) in test_cases {
            let session = PokerSession {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                session_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                duration_minutes: minutes,
                buy_in_amount: BigDecimal::from_f64(100.0).unwrap(),
                rebuy_amount: BigDecimal::from_f64(0.0).unwrap(),
                cash_out_amount: BigDecimal::from_f64(100.0).unwrap(),
                notes: None,
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            };

            let csv = generate_csv(&[session]);
            let lines: Vec<&str> = csv.lines().collect();
            assert!(
                lines[1].contains(expected_hours),
                "Expected {} hours for {} minutes, got: {}",
                expected_hours,
                minutes,
                lines[1]
            );
        }
    }

    // Property-based tests for CSV escaping
    proptest! {
        #[test]
        fn field_without_special_chars_unchanged(s in "[a-zA-Z0-9 ]{0,100}") {
            // Fields without commas, quotes, or newlines should remain unchanged
            let result = escape_csv_field(&s);
            prop_assert_eq!(result, s);
        }

        #[test]
        fn field_with_comma_gets_quoted(
            prefix in "[a-zA-Z0-9]{0,20}",
            suffix in "[a-zA-Z0-9]{0,20}",
        ) {
            let input = format!("{},{}", prefix, suffix);
            let result = escape_csv_field(&input);
            prop_assert!(result.starts_with('"'), "Result should start with quote: {}", result);
            prop_assert!(result.ends_with('"'), "Result should end with quote: {}", result);
            // The inner content should have the comma
            prop_assert!(result.contains(','));
        }

        #[test]
        fn field_with_newline_gets_quoted(
            prefix in "[a-zA-Z0-9]{0,20}",
            suffix in "[a-zA-Z0-9]{0,20}",
        ) {
            let input = format!("{}\n{}", prefix, suffix);
            let result = escape_csv_field(&input);
            prop_assert!(result.starts_with('"'), "Result should start with quote: {}", result);
            prop_assert!(result.ends_with('"'), "Result should end with quote: {}", result);
        }

        #[test]
        fn field_with_quotes_gets_doubled(
            prefix in "[a-zA-Z0-9]{0,20}",
            middle in "[a-zA-Z0-9]{0,20}",
            suffix in "[a-zA-Z0-9]{0,20}",
        ) {
            let input = format!("{}\"{}\"{}",prefix, middle, suffix);
            let result = escape_csv_field(&input);
            // Should be wrapped in quotes
            prop_assert!(result.starts_with('"'));
            prop_assert!(result.ends_with('"'));
            // Internal quotes should be doubled
            let inner = &result[1..result.len()-1];
            prop_assert!(inner.contains("\"\""), "Internal quotes should be doubled: {}", result);
        }

        #[test]
        fn escaped_field_preserves_content_semantically(s in "[ -~]{0,50}") {
            // ASCII printable characters
            let result = escape_csv_field(&s);
            // The content should be recoverable
            if result.starts_with('"') && result.ends_with('"') {
                let inner = &result[1..result.len()-1];
                let unescaped = inner.replace("\"\"", "\"");
                prop_assert_eq!(unescaped, s.clone(), "Content not preserved for input: {:?}", s);
            } else {
                prop_assert_eq!(result, s.clone(), "Non-quoted content should match");
            }
        }

        #[test]
        fn multiple_commas_all_preserved(count in 1..=5_usize) {
            let input: String = (0..count).map(|_| "a,").collect();
            let result = escape_csv_field(&input);
            // Count commas in result (excluding wrapper quotes)
            let inner = &result[1..result.len()-1];
            let comma_count = inner.matches(',').count();
            prop_assert_eq!(comma_count, count, "All commas should be preserved");
        }

        #[test]
        fn multiple_quotes_all_doubled(count in 1..=5_usize) {
            let input: String = (0..count).map(|_| "\"").collect();
            let result = escape_csv_field(&input);
            // Should be wrapped, and each quote doubled
            // Input of n quotes becomes: "quote quote ... quote" with each quote doubled
            let inner = &result[1..result.len()-1];
            let doubled_count = inner.matches("\"\"").count();
            prop_assert_eq!(doubled_count, count, "All quotes should be doubled");
        }
    }

    // Property-based tests for duration to hours conversion
    proptest! {
        #[test]
        fn duration_conversion_is_correct(minutes in 1..=10000_i32) {
            let expected_hours = minutes as f64 / 60.0;
            let session = PokerSession {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                session_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                duration_minutes: minutes,
                buy_in_amount: BigDecimal::from_f64(100.0).unwrap(),
                rebuy_amount: BigDecimal::from_f64(0.0).unwrap(),
                cash_out_amount: BigDecimal::from_f64(100.0).unwrap(),
                notes: None,
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            };

            let csv = generate_csv(&[session]);
            let lines: Vec<&str> = csv.lines().collect();

            // The formatted hours should be close to expected
            let formatted = format!("{:.1}", expected_hours);
            prop_assert!(lines[1].contains(&formatted),
                "Expected {} for {} minutes, line: {}",
                formatted, minutes, lines[1]);
        }
    }
}
