use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;
use chrono::Utc;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    let symbol = "GTX-PERP";
    
    // Check if exists
    let exists = sqlx::query!("SELECT id FROM futures_products WHERE symbol = $1", symbol)
        .fetch_optional(&pool)
        .await?;

    if let Some(record) = exists {
        println!("Product {} already exists with ID: {}", symbol, record.id);
        return Ok(());
    }

    let product_id = Uuid::new_v4();
    let id = sqlx::query!(
        r#"
        INSERT INTO futures_products (id, symbol, base_asset, quote_asset, contract_size, expiration_date, current_price, is_active, created_at, updated_at)
        VALUES ($1, $2, 'GTX', 'USDC', 1.0, $3, 100.0, true, $4, $4)
        RETURNING id
        "#,
        product_id,
        symbol,
        Utc::now() + chrono::Duration::days(365),
        Utc::now()
    )
    .fetch_one(&pool)
    .await?
    .id;

    println!("Seeded Futures Product: {} (ID: {})", symbol, id);
    Ok(())
}
