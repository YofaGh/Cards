use sqlx::{migrate, postgres::PgPoolOptions, Error as SqlxError};

use crate::prelude::*;

pub async fn create_database_pool() -> Result<PgPool> {
    let config: &'static Config = get_config();
    let pool: Result<PgPool> = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&config.database.url)
        .await
        .map_err(|err: SqlxError| Error::Other(format!("Failed to connect to database: {err}")));
    println!("Connected to database successfully");
    pool
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|err: migrate::MigrateError| {
            Error::Other(format!("Failed to run migrations: {err}"))
        })?;
    println!("Migrations completed successfully");
    Ok(())
}

pub async fn test_database_connection(pool: &PgPool) -> Result<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|err: SqlxError| {
            Error::Other(format!("Database connection test failed: {err}"))
        })?;
    println!("Database connection test succeeded");
    Ok(())
}
