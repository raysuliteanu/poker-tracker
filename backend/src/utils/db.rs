use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool, PooledConnection};
use std::env;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

/// Trait for providing database connections.
/// Returns pooled connections with boxed errors for maximum flexibility.
/// Used by both production code and tests.
pub trait DbProvider: Send + Sync {
    fn get_connection(&self) -> Result<DbConnection, Box<dyn std::error::Error + Send + Sync>>;
}

/// Production implementation using a connection pool
impl DbProvider for DbPool {
    fn get_connection(&self) -> Result<DbConnection, Box<dyn std::error::Error + Send + Sync>> {
        self.get()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

pub fn establish_connection_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database connection pool")
}
