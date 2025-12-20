use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool, PooledConnection};
use std::env;
use std::sync::Arc;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
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
    type Connection = DbConnection;
    type Error = r2d2::PoolError;

    fn get_connection(&self) -> Result<Self::Connection, Self::Error> {
        self.get()
    }
}

pub trait PooledConnectionProvider: Send + Sync {
    fn get_connection(&self) -> Result<DbConnection, Box<dyn std::error::Error + Send + Sync>>;
}

impl PooledConnectionProvider for DbPool {
    fn get_connection(&self) -> Result<DbConnection, Box<dyn std::error::Error + Send + Sync>> {
        self.get()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

impl DbConnectionProvider for Arc<dyn PooledConnectionProvider> {
    type Connection = DbConnection;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn get_connection(&self) -> Result<Self::Connection, Self::Error> {
        (**self).get_connection()
    }
}

pub fn establish_connection_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database connection pool")
}
