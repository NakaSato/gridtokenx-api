use api_gateway::config::SolanaProgramsConfig;
use api_gateway::services::BlockchainService;
use dotenvy::dotenv;
use solana_sdk::signature::Signer;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    // Setup logging
    tracing_subscriber::fmt::init();
    
    // Load Authority
    let authority_path = std::env::var("AUTHORITY_WALLET_PATH").unwrap_or("../keypairs/dev-wallet.json".to_string());
    println!("Loading authority from: {}", authority_path);
    let authority = BlockchainService::load_keypair_from_file(&authority_path)
        .expect("Failed to load authority");
        
    println!("Authority: {}", authority.pubkey());
    
    // Config
    let rpc_url = env::var("SOLANA_RPC_URL").unwrap_or("http://localhost:8899".to_string());
    let program_config = SolanaProgramsConfig::default();
    
    // Initialize Blockchain Service
    let blockchain = BlockchainService::new(rpc_url.clone(), "localnet".to_string(), program_config)
        .expect("Failed to init blockchain service");

    println!("Initializing Blockchain on {}...", rpc_url);

    // 1. Initialize Registry
    println!("1. Initializing Registry...");
    match blockchain.initialize_registry(&authority).await {
        Ok(sig) => println!("   Success: {}", sig),
        Err(e) => println!("   Failed (maybe already init): {}", e),
    }

    // 2. Initialize Oracle
    // Note: We need API Gateway Pubkey. Using Authority as API Gateway for testing.
    println!("2. Initializing Oracle...");
    match blockchain.initialize_oracle(&authority, &authority.pubkey()).await {
        Ok(sig) => println!("   Success: {}", sig),
        Err(e) => println!("   Failed (maybe already init): {}", e),
    }

    // 3. Initialize Governance
    println!("3. Initializing Governance...");
    match blockchain.initialize_governance(&authority).await {
        Ok(sig) => println!("   Success: {}", sig),
        Err(e) => println!("   Failed (maybe already init): {}", e),
    }

    // 4. Initialize Energy Token
    println!("4. Initializing Energy Token Mint...");
    match blockchain.initialize_energy_token(&authority).await {
        Ok(sig) => println!("   Success: {}", sig),
        Err(e) => println!("   Failed (maybe already init): {}", e),
    }

    // 5. Initialize Trading Market
    println!("5. Initializing Trading Market...");
    match blockchain.initialize_trading_market(&authority).await {
        Ok(sig) => println!("   Success: {}", sig),
        Err(e) => println!("   Failed (maybe already init): {}", e),
    }
    
    println!("Blockchain Initialization Complete.");
    Ok(())
}
