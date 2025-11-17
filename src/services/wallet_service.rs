use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
};
use solana_client::rpc_client::RpcClient;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, error, warn};

/// Service for managing Solana wallets in development environment
#[derive(Clone)]
pub struct WalletService {
    rpc_client: Arc<RpcClient>,
    /// The authority keypair (cached in memory)
    authority_keypair: Arc<RwLock<Option<Arc<Keypair>>>>,
    /// Path to wallet file (if loading from file)
    wallet_path: Option<String>,
}

impl WalletService {
    /// Create a new WalletService instance
    pub fn new(rpc_url: &str) -> Self {
        info!("Initializing WalletService with RPC URL: {}", rpc_url);
        Self {
            rpc_client: Arc::new(RpcClient::new(rpc_url.to_string())),
            authority_keypair: Arc::new(RwLock::new(None)),
            wallet_path: None,
        }
    }

    /// Create wallet service with a specific wallet file path
    pub fn with_path(rpc_url: &str, wallet_path: String) -> Self {
        info!("Initializing WalletService with RPC URL: {} and wallet path: {}", rpc_url, wallet_path);
        Self {
            rpc_client: Arc::new(RpcClient::new(rpc_url.to_string())),
            authority_keypair: Arc::new(RwLock::new(None)),
            wallet_path: Some(wallet_path),
        }
    }

    /// Create a new Solana keypair for development
    pub fn create_keypair() -> Keypair {
        let keypair = Keypair::new();
        info!("Created new keypair with pubkey: {}", keypair.pubkey());
        keypair
    }

    /// Get wallet balance in lamports
    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        match self.rpc_client.get_balance(pubkey) {
            Ok(balance) => {
                info!("Retrieved balance for {}: {} lamports", pubkey, balance);
                Ok(balance)
            }
            Err(e) => {
                error!("Failed to get balance for {}: {}", pubkey, e);
                Err(e.into())
            }
        }
    }

    /// Request airdrop for development (localhost only)
    pub async fn request_airdrop(&self, pubkey: &Pubkey, amount_sol: f64) -> Result<Signature> {
        let lamports = (amount_sol * 1_000_000_000.0) as u64; // Convert SOL to lamports
        
        info!("Requesting airdrop of {} SOL ({} lamports) for {}", amount_sol, lamports, pubkey);
        
        match self.rpc_client.request_airdrop(pubkey, lamports) {
            Ok(signature) => {
                info!("Airdrop successful. Signature: {}", signature);
                
                // Wait for confirmation in development
                let _ = self.confirm_transaction(&signature).await;
                
                Ok(signature)
            }
            Err(e) => {
                error!("Airdrop failed for {}: {}", pubkey, e);
                Err(e.into())
            }
        }
    }

    /// Confirm transaction (for development)
    pub async fn confirm_transaction(&self, signature: &Signature) -> Result<bool> {
        // Simple confirmation check for development
        // In production, you'd want more sophisticated confirmation logic
        match self.rpc_client.get_signature_status(signature) {
            Ok(Some(_)) => {
                info!("Transaction {} confirmed", signature);
                Ok(true)
            }
            Ok(None) => {
                info!("Transaction {} not yet confirmed", signature);
                Ok(false)
            }
            Err(e) => {
                error!("Error checking transaction status: {}", e);
                Err(e.into())
            }
        }
    }

    /// Validate Solana address format
    pub fn is_valid_address(address: &str) -> bool {
        match Pubkey::from_str(address) {
            Ok(_) => {
                info!("Valid Solana address: {}", address);
                true
            }
            Err(_) => {
                info!("Invalid Solana address: {}", address);
                false
            }
        }
    }

    /// Get recent blockhash (useful for transaction building)
    pub async fn get_recent_blockhash(&self) -> Result<solana_sdk::hash::Hash> {
        match self.rpc_client.get_latest_blockhash() {
            Ok(blockhash) => {
                info!("Retrieved recent blockhash: {}", blockhash);
                Ok(blockhash)
            }
            Err(e) => {
                error!("Failed to get recent blockhash: {}", e);
                Err(e.into())
            }
        }
    }

    /// Check if RPC connection is healthy
    pub async fn health_check(&self) -> Result<bool> {
        match self.rpc_client.get_health() {
            Ok(_) => {
                info!("Solana RPC health check passed");
                Ok(true)
            }
            Err(e) => {
                error!("Solana RPC health check failed: {}", e);
                Err(e.into())
            }
        }
    }

    // ====================================================================
    // Authority Keypair Management (Phase 4)
    // ====================================================================

    /// Load authority keypair from file
    /// File should be a JSON array of numbers (standard Solana keypair format)
    pub async fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_ref = path.as_ref();
        info!("Loading authority keypair from: {:?}", path_ref);

        // Read file contents
        let contents = fs::read_to_string(path_ref)
            .with_context(|| format!("Failed to read wallet file: {:?}", path_ref))?;

        // Parse JSON array of bytes
        let keypair_bytes: Vec<u8> = serde_json::from_str(&contents)
            .with_context(|| "Failed to parse wallet file as JSON array")?;

        if keypair_bytes.len() != 64 {
            return Err(anyhow!(
                "Invalid keypair file: expected 64 bytes, got {}",
                keypair_bytes.len()
            ));
        }

        // Solana SDK 3.0 uses new_from_array with [u8; 32] (secret key only)
        // The first 32 bytes are the secret key
        let secret_key: [u8; 32] = keypair_bytes[..32].try_into()
            .map_err(|_| anyhow!("Failed to extract secret key"))?;
        
        let keypair = Keypair::new_from_array(secret_key);

        info!("Successfully loaded authority keypair: {}", keypair.pubkey());

        // Cache in memory
        let mut lock = self.authority_keypair.write().await;
        *lock = Some(Arc::new(keypair));

        Ok(())
    }

    /// Load authority keypair from environment variable
    /// Expects base58-encoded private key in AUTHORITY_WALLET_PRIVATE_KEY
    pub async fn load_from_env(&self) -> Result<()> {
        info!("Loading authority keypair from environment variable");

        let private_key_str = std::env::var("AUTHORITY_WALLET_PRIVATE_KEY")
            .with_context(|| "AUTHORITY_WALLET_PRIVATE_KEY environment variable not set")?;

        // Decode base58
        let keypair_bytes = bs58::decode(&private_key_str)
            .into_vec()
            .with_context(|| "Failed to decode base58 private key")?;

        if keypair_bytes.len() != 64 {
            return Err(anyhow!(
                "Invalid private key: expected 64 bytes, got {}",
                keypair_bytes.len()
            ));
        }

        // Solana SDK 3.0 uses new_from_array with [u8; 32] (secret key only)
        // The first 32 bytes are the secret key
        let secret_key: [u8; 32] = keypair_bytes[..32].try_into()
            .map_err(|_| anyhow!("Failed to extract secret key"))?;
        
        let keypair = Keypair::new_from_array(secret_key);

        info!("Successfully loaded authority keypair from env: {}", keypair.pubkey());

        // Cache in memory
        let mut lock = self.authority_keypair.write().await;
        *lock = Some(Arc::new(keypair));

        Ok(())
    }

    /// Initialize wallet from configuration
    /// Tries file first, then environment variable
    pub async fn initialize_authority(&self) -> Result<()> {
        // Try loading from configured file path
        if let Some(ref path) = self.wallet_path {
            if Path::new(path).exists() {
                return self.load_from_file(path).await;
            } else {
                warn!("Wallet file not found: {}", path);
            }
        }

        // Try loading from default locations
        let default_paths = vec![
            "./dev-wallet.json",
            "../dev-wallet.json",
            "/app/dev-wallet.json",
        ];

        for path in default_paths {
            if Path::new(path).exists() {
                debug!("Found wallet file at: {}", path);
                return self.load_from_file(path).await;
            }
        }

        // Try environment variable
        if std::env::var("AUTHORITY_WALLET_PRIVATE_KEY").is_ok() {
            return self.load_from_env().await;
        }

        Err(anyhow!(
            "No authority wallet found. Set AUTHORITY_WALLET_PRIVATE_KEY env var or provide dev-wallet.json"
        ))
    }

    /// Get the authority keypair
    /// Returns error if not loaded
    pub async fn get_authority_keypair(&self) -> Result<Arc<Keypair>> {
        let lock = self.authority_keypair.read().await;
        lock.as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("Authority keypair not loaded. Call initialize_authority() first."))
    }

    /// Get the authority public key as a string
    pub async fn get_authority_pubkey_string(&self) -> Result<String> {
        let keypair = self.get_authority_keypair().await?;
        Ok(keypair.pubkey().to_string())
    }

    /// Check if authority wallet is loaded
    pub async fn is_authority_loaded(&self) -> bool {
        let lock = self.authority_keypair.read().await;
        lock.is_some()
    }

    /// Save keypair to file (for testing/development)
    #[cfg(test)]
    pub fn save_keypair_to_file<P: AsRef<Path>>(keypair: &Keypair, path: P) -> Result<()> {
        let bytes = keypair.to_bytes();
        let json = serde_json::to_string(&bytes.to_vec())?;
        fs::write(path, json)?;
        Ok(())
    }
}

/// Helper function to convert lamports to SOL
pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1_000_000_000.0
}

/// Helper function to convert SOL to lamports
pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * 1_000_000_000.0) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_keypair() {
        let keypair = WalletService::create_keypair();
        assert!(!keypair.pubkey().to_string().is_empty());
    }

    #[test]
    fn test_is_valid_address() {
        // Test valid address
        let valid_address = "11111111111111111111111111111112"; // System program ID
        assert!(WalletService::is_valid_address(valid_address));

        // Test invalid address
        let invalid_address = "invalid_address";
        assert!(!WalletService::is_valid_address(invalid_address));
    }

    #[test]
    fn test_lamports_conversion() {
        assert_eq!(lamports_to_sol(1_000_000_000), 1.0);
        assert_eq!(sol_to_lamports(1.0), 1_000_000_000);
    }

    #[tokio::test]
    async fn test_load_authority_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let wallet_path = temp_dir.path().join("test-wallet.json");

        // Generate and save a test keypair
        let keypair = Keypair::new();
        let expected_pubkey = keypair.pubkey();
        WalletService::save_keypair_to_file(&keypair, &wallet_path).unwrap();

        // Load it
        let wallet_service = WalletService::new("http://localhost:8899");
        wallet_service.load_from_file(&wallet_path).await.unwrap();

        // Verify
        let loaded_keypair = wallet_service.get_authority_keypair().await.unwrap();
        assert_eq!(loaded_keypair.pubkey(), expected_pubkey);
    }

    #[tokio::test]
    async fn test_is_authority_loaded() {
        let wallet_service = WalletService::new("http://localhost:8899");
        assert!(!wallet_service.is_authority_loaded().await);

        let temp_dir = TempDir::new().unwrap();
        let wallet_path = temp_dir.path().join("test-wallet.json");
        let keypair = Keypair::new();
        WalletService::save_keypair_to_file(&keypair, &wallet_path).unwrap();

        wallet_service.load_from_file(&wallet_path).await.unwrap();
        assert!(wallet_service.is_authority_loaded().await);
    }

    #[tokio::test]
    async fn test_get_authority_pubkey_string() {
        let temp_dir = TempDir::new().unwrap();
        let wallet_path = temp_dir.path().join("test-wallet.json");

        let keypair = Keypair::new();
        let expected_pubkey = keypair.pubkey().to_string();
        WalletService::save_keypair_to_file(&keypair, &wallet_path).unwrap();

        let wallet_service = WalletService::new("http://localhost:8899");
        wallet_service.load_from_file(&wallet_path).await.unwrap();

        let pubkey_string = wallet_service.get_authority_pubkey_string().await.unwrap();
        assert_eq!(pubkey_string, expected_pubkey);
    }

    #[tokio::test]
    async fn test_get_keypair_before_load() {
        let wallet_service = WalletService::new("http://localhost:8899");
        let result = wallet_service.get_authority_keypair().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not loaded"));
    }
}