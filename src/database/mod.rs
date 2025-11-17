use anyhow::Result;
use sqlx::{PgPool, Pool, Postgres, postgres::PgPoolOptions};
use tracing::{info, warn};
use std::time::Duration;

pub mod schema;

pub type DatabasePool = Pool<Postgres>;

pub async fn setup_database(database_url: &str) -> Result<DatabasePool> {
    info!("Connecting to database with enhanced security settings");
    
    // Parse database URL and check for SSL parameters
    let ssl_mode = if database_url.contains("sslmode=require") || 
                      database_url.contains("sslmode=verify-ca") ||
                      database_url.contains("sslmode=verify-full") {
        "SSL enabled"
    } else {
        warn!("Database connection does not enforce SSL. Consider adding sslmode=require to connection string");
        "SSL not enforced"
    };
    
    info!("Database SSL mode: {}", ssl_mode);
    
    // Create connection pool with security-focused settings
    let pool = PgPoolOptions::new()
        .max_connections(20) // Limit maximum connections
        .min_connections(2)  // Maintain minimum pool
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600)) // Close idle connections after 10 minutes
        .max_lifetime(Duration::from_secs(1800)) // Recycle connections after 30 minutes
        .connect(database_url)
        .await?;
    
    // Test the connection
    sqlx::query("SELECT 1").execute(&pool).await?;
    
    info!("✅ Database connection established successfully");
    
    Ok(pool)
}

pub async fn setup_timescale_database(influxdb_url: &str) -> Result<Option<DatabasePool>> {
    // TODO: Implement proper InfluxDB client integration
    // For now, we skip InfluxDB connection if it's an HTTP URL
    if influxdb_url.starts_with("http://") || influxdb_url.starts_with("https://") {
        info!("InfluxDB connection skipped (HTTP URL detected): {}", influxdb_url);
        info!("Note: InfluxDB integration requires proper client library - currently not in use");
        return Ok(None);
    }
    
    info!("Connecting to TimescaleDB: {}", influxdb_url);
    
    let pool = PgPool::connect(influxdb_url).await?;
    
    // Test the connection
    sqlx::query("SELECT 1").execute(&pool).await?;
    
    Ok(Some(pool))
}

pub async fn run_migrations(pool: &DatabasePool) -> Result<()> {
    info!("Running database migrations");
    
    sqlx::migrate!("./migrations").run(pool).await?;
    
    info!("Database migrations completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // Note: Testcontainers integration will be implemented in Phase 2
    // when we set up full integration testing

    pub struct TestDatabase {
        pub pool: DatabasePool,
    }

    impl TestDatabase {
        pub async fn new() -> Result<Self> {
            // For now, just create a mock connection
            // In Phase 2, we'll implement proper test database setup
            todo!("Test database setup will be implemented in Phase 2")
        }
    }
}