// @generated automatically by Diesel CLI.

diesel::table! {
    poker_sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        session_date -> Date,
        duration_minutes -> Int4,
        buy_in_amount -> Numeric,
        rebuy_amount -> Numeric,
        cash_out_amount -> Numeric,
        notes -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        username -> Varchar,
        password_hash -> Varchar,
        cookie_consent -> Bool,
        cookie_consent_date -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(poker_sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    poker_sessions,
    users,
);
