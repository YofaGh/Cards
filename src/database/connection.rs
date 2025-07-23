use sqlx::{postgres::PgPoolOptions, Error as SqlxError};

use crate::prelude::*;

pub async fn create_database_pool() -> Result<PgPool> {
    let config: &'static Config = get_config();
    PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&config.database.url)
        .await
        .map_err(|e: SqlxError| Error::Other(format!("Failed to connect to database: {}", e)))
}

pub async fn test_database_connection(pool: &PgPool) -> Result<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e: SqlxError| Error::Other(format!("Database connection test failed: {}", e)))?;
    Ok(())
}
