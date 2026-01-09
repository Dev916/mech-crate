//! Migrate CLI Command
//!
//! Run database migrations.

use sqlx::migrate::MigrateDatabase;
use std::env;

pub async fn run() -> anyhow::Result<()> {
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("DATABASE_URL environment variable not set"))?;

    tracing::info!("Running migrations...");

    // Create database if it doesn't exist
    if !sqlx::Postgres::database_exists(&database_url).await? {
        tracing::info!("Creating database...");
        sqlx::Postgres::create_database(&database_url).await?;
    }

    // Connect and run migrations
    let pool = sqlx::PgPool::connect(&database_url).await?;
    
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    tracing::info!("✓ Migrations complete");

    Ok(())
}
