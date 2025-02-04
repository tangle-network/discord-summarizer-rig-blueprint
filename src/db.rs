use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn setup_database() -> Result<PgPool, Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Configure and create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await?;

    // Create tables if they don't exist
    create_tables(&pool).await?;

    Ok(pool)
}

async fn create_tables(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // Create messages table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id SERIAL PRIMARY KEY,
            data JSONB NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#,
    )
    .execute(pool)
    .await?;

    // Create summaries table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS summaries (
            id SERIAL PRIMARY KEY,
            summary VARCHAR NOT NULL,
            date DATE NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
