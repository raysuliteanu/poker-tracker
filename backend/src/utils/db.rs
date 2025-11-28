use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool, PooledConnection};
use std::env;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
#[allow(dead_code)]
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

/// Trait for providing database connections
/// This allows us to use both pooled connections (in production)
/// and direct connections (in tests) with the same handler code
pub trait DbConnectionProvider {
    type Connection;
    type Error;

    fn get_connection(&self) -> Result<Self::Connection, Self::Error>;
}

/// Production implementation using a connection pool
impl DbConnectionProvider for DbPool {
    type Connection = PooledConnection<ConnectionManager<PgConnection>>;
    type Error = r2d2::PoolError;

    fn get_connection(&self) -> Result<Self::Connection, Self::Error> {
        self.get()
    }
}

pub fn establish_connection_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database connection pool")
}
