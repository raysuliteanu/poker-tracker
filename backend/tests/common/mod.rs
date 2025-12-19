use bcrypt::{DEFAULT_COST, hash};
use diesel::PgConnection;
use diesel::prelude::*;
use poker_tracker::models::user::{NewUser, User};
use poker_tracker::models::{CreatePokerSessionRequest, PokerSession};
use poker_tracker::schema::{poker_sessions, users};
use poker_tracker::utils::DbConnectionProvider;
use testcontainers::ContainerAsync;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use uuid::Uuid;

/// A struct that manages a temporary PostgreSQL container and its connection URL.
pub struct TestDb {
    pub database_url: String,
    // The container handle must be held for the database to stay alive
    #[expect(dead_code)]
    container: ContainerAsync<Postgres>,
}

impl TestDb {
    /// Starts a new Postgres container, runs migrations, and returns the setup.
    pub async fn new() -> Self {
        let container = Postgres::default().start().await.unwrap();

        let host = container.get_host().await.unwrap();
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let database_url = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            host, host_port
        );

        TestDb::run_migrations(&database_url).expect("Failed to run migrations on test DB");

        Self {
            database_url,
            container,
        }
    }

    /// Connects to the DB and applies all pending migrations.
    fn run_migrations(url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        use diesel_migrations::{MigrationHarness, embed_migrations};

        // You must have a `migrations` directory in your project root
        const MIGRATIONS: diesel_migrations::EmbeddedMigrations = embed_migrations!();

        let mut connection = PgConnection::establish(url)?;
        connection.run_pending_migrations(MIGRATIONS)?;

        Ok(())
    }

    /// Create a connection pool from this test database
    pub fn create_pool(&self) -> poker_tracker::utils::DbPool {
        use diesel::r2d2::{ConnectionManager, Pool};
        let manager = ConnectionManager::<PgConnection>::new(&self.database_url);
        Pool::builder()
            .build(manager)
            .expect("Failed to create test database pool")
    }
}

impl poker_tracker::utils::DbConnectionProvider for TestDb {
    type Connection = PgConnection;
    type Error = diesel::ConnectionError;

    fn get_connection(&self) -> Result<Self::Connection, Self::Error> {
        PgConnection::establish(&self.database_url)
    }
}

/// Helper to create a test user directly in the database (without password hashing)
pub fn create_test_user_raw(db: &TestDb, email: &str, username: &str) -> User {
    let mut conn = db.get_connection().expect("Failed to get db connection");
    let new_user = NewUser {
        email: email.to_string(),
        username: username.to_string(),
        password_hash: "raw_hash_for_testing".to_string(),
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(&mut conn)
        .expect("Failed to create test user")
}

/// Helper to create a test user with a properly hashed password
pub fn create_test_user_with_password(
    db: &TestDb,
    email: &str,
    username: &str,
    password: &str,
) -> User {
    let mut conn = db.get_connection().expect("Failed to get db connection");
    let password_hash = hash(password, DEFAULT_COST).expect("Failed to hash password");
    let new_user = NewUser {
        email: email.to_string(),
        username: username.to_string(),
        password_hash,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(&mut conn)
        .expect("Failed to create test user")
}

/// Helper to get a user by email
pub fn get_user_by_email(db: &TestDb, email: &str) -> Option<User> {
    let mut conn = db.get_connection().expect("Failed to get db connection");
    users::table
        .filter(users::email.eq(email))
        .first::<User>(&mut conn)
        .ok()
}

/// Helper to get a user by ID
pub fn get_user_by_id(db: &TestDb, user_id: Uuid) -> Option<User> {
    let mut conn = db.get_connection().expect("Failed to get db connection");
    users::table.find(user_id).first::<User>(&mut conn).ok()
}

/// Helper to create a default session request for testing
pub fn default_session_request() -> CreatePokerSessionRequest {
    CreatePokerSessionRequest {
        session_date: "2024-01-15".to_string(),
        duration_minutes: 120,
        buy_in_amount: 100.0,
        rebuy_amount: Some(50.0),
        cash_out_amount: 200.0,
        notes: Some("Test session".to_string()),
    }
}

/// Helper to get all sessions for a user
pub fn get_sessions_for_user(db: &TestDb, user_id: Uuid) -> Vec<PokerSession> {
    let mut conn = db.get_connection().expect("Failed to get db connection");
    poker_sessions::table
        .filter(poker_sessions::user_id.eq(user_id))
        .order(poker_sessions::session_date.desc())
        .load::<PokerSession>(&mut conn)
        .expect("Failed to load sessions")
}

/// Helper to get a session by ID
pub fn get_session_by_id(db: &TestDb, session_id: Uuid) -> Option<PokerSession> {
    let mut conn = db.get_connection().expect("Failed to get db connection");
    poker_sessions::table
        .find(session_id)
        .first::<PokerSession>(&mut conn)
        .ok()
}

/// Helper to delete a session by ID (returns number of rows deleted)
pub fn delete_session_by_id(db: &TestDb, session_id: Uuid, user_id: Uuid) -> usize {
    let mut conn = db.get_connection().expect("Failed to get db connection");
    diesel::delete(
        poker_sessions::table
            .filter(poker_sessions::id.eq(session_id))
            .filter(poker_sessions::user_id.eq(user_id)),
    )
    .execute(&mut conn)
    .expect("Failed to delete session")
}

pub(crate) mod fixtures {
    use crate::common::TestDb;
    use rstest::fixture;

    #[fixture]
    pub async fn test_db() -> TestDb {
        TestDb::new().await
    }
}
