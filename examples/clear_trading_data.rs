use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    println!("Connecting to database to clear trading data...");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    println!("Clearing settlements...");
    sqlx::query("TRUNCATE TABLE settlements CASCADE").execute(&pool).await?;
    
    println!("Clearing order_matches...");
    sqlx::query("TRUNCATE TABLE order_matches CASCADE").execute(&pool).await?;
    
    println!("Clearing trading_orders...");
    sqlx::query("TRUNCATE TABLE trading_orders CASCADE").execute(&pool).await?;
    
    println!("Clearing users (clean slate for seller/buyer)...");
    sqlx::query("DELETE FROM users WHERE username LIKE 'seller_%' OR username LIKE 'buyer_%'").execute(&pool).await?;

    println!("✅ Database cleared successfully!");
    Ok(())
}
