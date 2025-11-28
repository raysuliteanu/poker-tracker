use diesel::PgConnection;
use diesel::prelude::*;
use testcontainers::ContainerAsync;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

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
}

impl poker_tracker::utils::DbConnectionProvider for TestDb {
    type Connection = PgConnection;
    type Error = diesel::ConnectionError;

    fn get_connection(&self) -> Result<Self::Connection, Self::Error> {
        PgConnection::establish(&self.database_url)
    }
}

pub(crate) mod fixtures {
    use crate::common::TestDb;
    use rstest::fixture;

    #[fixture]
    pub async fn test_db() -> TestDb {
        TestDb::new().await
    }
}
