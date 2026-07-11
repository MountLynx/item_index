use std::path::Path;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

/// Type alias for the database connection pool.
pub type DbPool = SqlitePool;

/// Create a new SQLite connection pool.
///
/// The database file is created if it doesn't exist (`mode=rwc`).
/// A maximum of 5 connections is maintained.
pub async fn create_pool(path: &Path) -> Result<DbPool, sqlx::Error> {
    let database_url = format!("sqlite:{}?mode=rwc", path.display());
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}

/// Run all pending migrations from the `migrations/` directory.
///
/// Migrations are embedded at compile time via `sqlx::migrate!`.
pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| sqlx::Error::Migrate(Box::new(e)))?;
    Ok(())
}
