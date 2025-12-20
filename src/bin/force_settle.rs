use api_gateway::config::SolanaProgramsConfig;
use api_gateway::services::{BlockchainService, WalletService};
use dotenvy::dotenv;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::env;
use sqlx::postgres::PgPoolOptions;
use base64::{engine::general_purpose, Engine as _};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    
    // Connect DB
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
        
    // Config
    let rpc_url = env::var("SOLANA_RPC_URL").unwrap_or("http://localhost:8899".to_string());
    let encryption_secret = env::var("ENCRYPTION_SECRET").expect("ENCRYPTION_SECRET missing");
    
    // Init Blockchain
    let blockchain = BlockchainService::new(
        rpc_url, 
        "localnet".to_string(), 
        SolanaProgramsConfig::default()
    )?;
    
    // Settlement ID
    let settlement_id = Uuid::parse_str("7c8d902c-f748-4c6b-af84-dd3abc71360f")?;
    
    // Fetch Settlement Data
    let settlement = sqlx::query!(
        "SELECT seller_id, buyer_id, energy_amount FROM settlements WHERE id = $1", 
        settlement_id
    )
    .fetch_one(&pool)
    .await?;
    
    // Fetch Seller Keys
    let seller_row = sqlx::query!(
        "SELECT encrypted_private_key, wallet_salt, encryption_iv, wallet_address FROM users WHERE id = $1",
        settlement.seller_id
    )
    .fetch_one(&pool)
    .await?;
    
    let encrypted_pk = seller_row.encrypted_private_key.ok_or("No private key")?;
    let encrypted_b64 = general_purpose::STANDARD.encode(&encrypted_pk);
    let salt_b64 = general_purpose::STANDARD.encode(&seller_row.wallet_salt.unwrap());
    let iv_b64 = general_purpose::STANDARD.encode(&seller_row.encryption_iv.unwrap());
    
    let decrypted = WalletService::decrypt_private_key(
        &encryption_secret,
        &encrypted_b64,
        &salt_b64,
        &iv_b64
    )?;
    
    let seller_keypair = if decrypted.len() == 32 {
        Keypair::new_from_array(decrypted[..32].try_into().unwrap())
    } else if decrypted.len() == 64 {
        let seed: [u8; 32] = decrypted[..32].try_into().unwrap();
        Keypair::new_from_array(seed)
    } else {
        panic!("Invalid key length: {}", decrypted.len());
    };
    
    println!("Seller Pubkey: {}", seller_keypair.pubkey());
    if seller_row.wallet_address.unwrap() != seller_keypair.pubkey().to_string() {
        panic!("Mismatch!");
    }
    
    // Buyer Wallet
    let buyer_row = sqlx::query!("SELECT wallet_address FROM users WHERE id = $1", settlement.buyer_id)
        .fetch_one(&pool).await?;
    let buyer_wallet = Pubkey::from_str(&buyer_row.wallet_address.unwrap())?;
    
    // Mint
    let mint_str = env::var("ENERGY_TOKEN_MINT").expect("ENERGY_TOKEN_MINT missing");
    let mint = Pubkey::from_str(&mint_str)?;
    
    // ATAs
    // Assuming ATAs exist (we checked/created them)
    let seller_ata = spl_associated_token_account::get_associated_token_address(&seller_keypair.pubkey(), &mint);
    let buyer_ata = spl_associated_token_account::get_associated_token_address(&buyer_wallet, &mint);
    
    println!("Seller ATA: {}", seller_ata);
    println!("Buyer ATA: {}", buyer_ata);
    
    // Amount
    // Logic: energy_amount * 1_000_000_000
    let amount_decimal = settlement.energy_amount * rust_decimal::Decimal::from(1_000_000_000);
    let amount_u64 = amount_decimal.trunc().to_string().parse::<u64>()?;
    println!("Transfer Amount: {}", amount_u64);
    
    // Transfer
    let sig = blockchain.transfer_tokens(
        &seller_keypair,
        &seller_ata,
        &buyer_ata,
        &mint,
        amount_u64,
        9
    ).await?;
    
    println!("Transfer Success! Sig: {}", sig);
    
    // Update DB
    sqlx::query!(
        "UPDATE settlements SET status = 'completed', transaction_hash = $1, processed_at = NOW(), updated_at = NOW() WHERE id = $2",
        sig.to_string(),
        settlement_id
    )
    .execute(&pool)
    .await?;
    
    println!("DB Updated.");
    Ok(())
}
