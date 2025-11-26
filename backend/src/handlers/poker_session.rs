use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    CreatePokerSessionRequest, NewPokerSession, PokerSession, SessionWithProfit,
    UpdatePokerSessionRequest, calculate_profit,
};
use crate::schema::poker_sessions;
use crate::utils::DbPool;

pub async fn create_session(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    session_req: web::Json<CreatePokerSessionRequest>,
) -> impl Responder {
    if let Err(errors) = session_req.validate() {
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

    let session_date = match NaiveDate::parse_from_str(&session_req.session_date, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid date format. Expected YYYY-MM-DD"
            }));
        }
    };

    let new_session = NewPokerSession {
        user_id,
        session_date,
        duration_minutes: session_req.duration_minutes,
        buy_in_amount: BigDecimal::from_f64(session_req.buy_in_amount).unwrap(),
        rebuy_amount: BigDecimal::from_f64(session_req.rebuy_amount.unwrap_or(0.0)).unwrap(),
        cash_out_amount: BigDecimal::from_f64(session_req.cash_out_amount).unwrap(),
        notes: session_req.notes.clone(),
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }));
        }
    };

    match diesel::insert_into(poker_sessions::table)
        .values(&new_session)
        .get_result::<PokerSession>(&mut conn)
    {
        Ok(session) => HttpResponse::Created().json(session),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create session: {}", e)
        })),
    }
}

pub async fn get_sessions(pool: web::Data<DbPool>, req: HttpRequest) -> impl Responder {
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
            HttpResponse::Ok().json(sessions_with_profit)
        }
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to fetch sessions"
        })),
    }
}

pub async fn get_session(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    session_id: web::Path<Uuid>,
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

    match poker_sessions::table
        .filter(poker_sessions::id.eq(session_id.into_inner()))
        .filter(poker_sessions::user_id.eq(user_id))
        .first::<PokerSession>(&mut conn)
    {
        Ok(session) => {
            let profit = calculate_profit(
                &session.buy_in_amount,
                &session.rebuy_amount,
                &session.cash_out_amount,
            );
            HttpResponse::Ok().json(SessionWithProfit { session, profit })
        }
        Err(_) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Session not found"
        })),
    }
}

pub async fn update_session(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    session_id: web::Path<Uuid>,
    update_req: web::Json<UpdatePokerSessionRequest>,
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

    // Verify ownership
    let existing_session = match poker_sessions::table
        .filter(poker_sessions::id.eq(session_id.into_inner()))
        .filter(poker_sessions::user_id.eq(user_id))
        .first::<PokerSession>(&mut conn)
    {
        Ok(s) => s,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Session not found"
            }));
        }
    };

    let session_date = if let Some(date_str) = &update_req.session_date {
        match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid date format. Expected YYYY-MM-DD"
                }));
            }
        }
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

    match diesel::update(poker_sessions::table.find(existing_session.id))
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
    {
        Ok(session) => HttpResponse::Ok().json(session),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to update session"
        })),
    }
}

pub async fn delete_session(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    session_id: web::Path<Uuid>,
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

    match diesel::delete(
        poker_sessions::table
            .filter(poker_sessions::id.eq(session_id.into_inner()))
            .filter(poker_sessions::user_id.eq(user_id)),
    )
    .execute(&mut conn)
    {
        Ok(count) if count > 0 => HttpResponse::Ok().json(serde_json::json!({
            "message": "Session deleted successfully"
        })),
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Session not found"
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to delete session"
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub time_range: Option<String>,
}

pub async fn export_sessions(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    query: web::Query<ExportQuery>,
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

    // Calculate cutoff date based on time range
    let cutoff_date = match query.time_range.as_deref() {
        Some("7days") => Some(Utc::now().naive_utc().date() - chrono::Duration::days(7)),
        Some("30days") => Some(Utc::now().naive_utc().date() - chrono::Duration::days(30)),
        Some("90days") => Some(Utc::now().naive_utc().date() - chrono::Duration::days(90)),
        Some("1year") => Some(Utc::now().naive_utc().date() - chrono::Duration::days(365)),
        Some("all") | None => None,
        Some(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid time_range. Valid options: 7days, 30days, 90days, 1year, all"
            }));
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

    HttpResponse::Ok()
        .content_type("text/csv; charset=utf-8")
        .insert_header((
            "Content-Disposition",
            format!(
                "attachment; filename=\"poker-sessions-{}.csv\"",
                query.time_range.as_deref().unwrap_or("all")
            ),
        ))
        .body(csv)
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
}
