use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires full stack running
    async fn test_complete_user_journey() -> Result<()> {
        let client = Client::new();
        let base_url = std::env::var("API_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        // Step 1: User Registration
        let register_response = register_user(&client, &base_url).await?;
        assert!(register_response.user_id.is_some());

        // Step 2: Email Verification (simulated)
        let user_id = register_response.user_id.unwrap();
        verify_email(&client, &base_url, &user_id).await?;

        // Step 3: Login
        let login_response = login_user(&client, &base_url).await?;
        assert!(!login_response.access_token.is_empty());

        // Step 4: Connect Wallet
        let wallet_response = connect_wallet(&client, &base_url, &login_response.access_token).await?;
        assert!(wallet_response.wallet_address.is_some());

        // Step 5: Verify Meter
        let meter_response = verify_meter(&client, &base_url, &login_response.access_token).await?;
        assert!(meter_response.meter_id.is_some());

        // Step 6: Submit Meter Reading
        let reading_response = submit_reading(&client, &base_url, &login_response.access_token).await?;
        assert!(reading_response.reading_id.is_some());

        // Step 7: Create Trading Order
        let order_response = create_order(&client, &base_url, &login_response.access_token).await?;
        assert!(order_response.order_id.is_some());

        // Step 8: Get Market Data
        let market_data = get_market_data(&client, &base_url, &login_response.access_token).await?;
        assert!(market_data.current_epoch.is_some());

        // Step 9: Get User Profile
        let profile = get_user_profile(&client, &base_url, &login_response.access_token).await?;
        assert_eq!(profile.user_id, user_id);

        println!("✅ Complete user journey test passed!");
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Requires admin setup
    async fn test_trading_workflow() -> Result<()> {
        let client = Client::new();
        let base_url = std::env::var("API_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        // Setup two users
        let seller_token = setup_trader(&client, &base_url, "seller").await?;
        let buyer_token = setup_trader(&client, &base_url, "buyer").await?;

        // Seller creates sell order
        let sell_order = create_sell_order(&client, &base_url, &seller_token).await?;
        assert!(sell_order.order_id.is_some());

        // Buyer creates buy order
        let buy_order = create_buy_order(&client, &base_url, &buyer_token).await?;
        assert!(buy_order.order_id.is_some());

        // Wait for market clearing (simulation)
        sleep(Duration::from_secs(5)).await;

        // Check if orders were matched
        let seller_orders = get_user_orders(&client, &base_url, &seller_token).await?;
        let buyer_orders = get_user_orders(&client, &base_url, &buyer_token).await?;

        // Verify order status updates
        let order_matched = seller_orders.iter().any(|o| o.status == "matched");
        assert!(order_matched, "Sell order should be matched");

        println!("✅ Trading workflow test passed!");
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Requires blockchain setup
    async fn test_blockchain_integration() -> Result<()> {
        let client = Client::new();
        let base_url = std::env::var("API_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        // Setup user with tokens
        let user_token = setup_trader(&client, &base_url, "blockchain_user").await?;
        
        // Submit meter reading
        let reading_response = submit_reading(&client, &base_url, &user_token).await?;
        let reading_id = reading_response.reading_id.unwrap();

        // Get admin token (assuming admin exists)
        let admin_token = get_admin_token(&client, &base_url).await?;

        // Mint tokens from reading
        let mint_response = mint_tokens_from_reading(&client, &base_url, &admin_token, &reading_id).await?;
        assert!(!mint_response.transaction_signature.is_empty());

        println!("✅ Blockchain integration test passed!");
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Requires validator setup
    async fn test_erc_certificate_flow() -> Result<()> {
        let client = Client::new();
        let base_url = std::env::var("API_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        // Setup REC authority
        let rec_token = setup_rec_authority(&client, &base_url).await?;

        // Setup regular user
        let user_token = setup_trader(&client, &base_url, "erc_user").await?;
        let user_profile = get_user_profile(&client, &base_url, &user_token).await?;

        // Issue ERC certificate
        let erc_request = ERCIssueRequest {
            user_id: user_profile.user_id,
            kwh_amount: 100.0,
            renewable_source: "Solar".to_string(),
            issuer_wallet: "test_issuer_wallet".to_string(),
        };

        let erc_response = issue_erc_certificate(&client, &base_url, &rec_token, erc_request).await?;
        assert!(erc_response.certificate_id.is_some());

        println!("✅ ERC certificate flow test passed!");
        Ok(())
    }
}

// Helper structs and functions

#[derive(Debug, Deserialize)]
struct RegisterResponse {
    user_id: Option<Uuid>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct WalletResponse {
    wallet_address: Option<String>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct MeterResponse {
    meter_id: Option<Uuid>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ReadingResponse {
    reading_id: Option<Uuid>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct OrderResponse {
    order_id: Option<Uuid>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct MarketDataResponse {
    current_epoch: Option<String>,
    orders: Vec<OrderInfo>,
}

#[derive(Debug, Deserialize)]
struct OrderInfo {
    order_id: Uuid,
    status: String,
    energy_amount: f64,
    price_per_kwh: f64,
}

#[derive(Debug, Deserialize)]
struct UserProfile {
    user_id: Uuid,
    email: String,
    name: String,
    wallet_address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MintResponse {
    transaction_signature: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ERCResponse {
    certificate_id: Option<String>,
    message: String,
}

#[derive(Debug, Serialize)]
struct ERCIssueRequest {
    user_id: Uuid,
    kwh_amount: f64,
    renewable_source: String,
    issuer_wallet: String,
}

async fn register_user(client: &Client, base_url: &str) -> Result<RegisterResponse> {
    let response = client
        .post(&format!("{}/api/auth/register", base_url))
        .json(&serde_json::json!({
            "email": "test@example.com",
            "password": "Test123!@#",
            "name": "Test User"
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response)
}

async fn verify_email(client: &Client, base_url: &str, user_id: &Uuid) -> Result<()> {
    // Simulate email verification (in real flow, would use email link)
    let response = client
        .post(&format!("{}/api/auth/verify-email", base_url))
        .json(&serde_json::json!({
            "token": format!("verify_token_{}", user_id)
        }))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

async fn login_user(client: &Client, base_url: &str) -> Result<LoginResponse> {
    let response = client
        .post(&format!("{}/api/auth/login", base_url))
        .json(&serde_json::json!({
            "email": "test@example.com",
            "password": "Test123!@#"
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response)
}

async fn connect_wallet(client: &Client, base_url: &str, token: &str) -> Result<WalletResponse> {
    let response = client
        .post(&format!("{}/api/user/wallet", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "wallet_address": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVLcdzNac"
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response)
}

async fn verify_meter(client: &Client, base_url: &str, token: &str) -> Result<MeterResponse> {
    let response = client
        .post(&format!("{}/api/meters/verify", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "meter_serial": "SM-2024-TEST001",
            "meter_key": "ABC123XYZ789",
            "verification_method": "serial",
            "manufacturer": Some("Test Manufacturer"),
            "meter_type": "residential",
            "location_address": Some("123 Test St")
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response)
}

async fn submit_reading(client: &Client, base_url: &str, token: &str) -> Result<ReadingResponse> {
    let response = client
        .post(&format!("{}/api/meters/submit-reading", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "meter_id": "550e8400-e29b-41d4-a716-446655440000", // Sample UUID
            "kwh_amount": 25.5,
            "reading_timestamp": chrono::Utc::now().to_rfc3339()
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response)
}

async fn create_order(client: &Client, base_url: &str, token: &str) -> Result<OrderResponse> {
    let response = client
        .post(&format!("{}/api/trading/orders", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "side": "sell",
            "energy_amount": 100.0,
            "price_per_kwh": 0.15,
            "expires_at": chrono::Utc::now() + chrono::Duration::hours(24)
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response)
}

async fn get_market_data(client: &Client, base_url: &str, token: &str) -> Result<MarketDataResponse> {
    let response = client
        .get(&format!("{}/api/market/data", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response)
}

async fn get_user_profile(client: &Client, base_url: &str, token: &str) -> Result<UserProfile> {
    let response = client
        .get(&format!("{}/api/user/profile", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response)
}

async fn setup_trader(client: &Client, base_url: &str, suffix: &str) -> Result<String> {
    let email = format!("trader_{}@example.com", suffix);
    let password = "Test123!@#";
    
    // Register
    client
        .post(&format!("{}/api/auth/register", base_url))
        .json(&serde_json::json!({
            "email": email,
            "password": password,
            "name": format!("Trader {}", suffix)
        }))
        .send()
        .await?
        .error_for_status()?;

    // Login
    let login_response: LoginResponse = client
        .post(&format!("{}/api/auth/login", base_url))
        .json(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // Connect wallet
    client
        .post(&format!("{}/api/user/wallet", base_url))
        .header("Authorization", format!("Bearer {}", login_response.access_token))
        .json(&serde_json::json!({
            "wallet_address": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVLcdzNac"
        }))
        .send()
        .await?
        .error_for_status()?;

    Ok(login_response.access_token)
}

async fn get_user_orders(client: &Client, base_url: &str, token: &str) -> Result<Vec<OrderInfo>> {
    let response = client
        .get(&format!("{}/api/trading/my-orders", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response)
}

async fn create_sell_order(client: &Client, base_url: &str, token: &str) -> Result<OrderResponse> {
    let response = client
        .post(&format!("{}/api/trading/orders", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "side": "sell",
            "energy_amount": 100.0,
            "price_per_kwh": 0.15,
            "expires_at": chrono::Utc::now() + chrono::Duration::hours(24)
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response)
}

async fn create_buy_order(client: &Client, base_url: &str, token: &str) -> Result<OrderResponse> {
    let response = client
        .post(&format!("{}/api/trading/orders", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "side": "buy",
            "energy_amount": 100.0,
            "price_per_kwh": 0.16,
            "expires_at": chrono::Utc::now() + chrono::Duration::hours(24)
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response)
}

async fn get_admin_token(client: &Client, base_url: &str) -> Result<String> {
    // Assuming admin user exists
    let response = client
        .post(&format!("{}/api/auth/login", base_url))
        .json(&serde_json::json!({
            "email": "admin@example.com",
            "password": "Admin123!@#"
        }))
        .send()
        .await?
        .error_for_status()?
        .json::<LoginResponse>()
        .await?;

    Ok(response.access_token)
}

async fn mint_tokens_from_reading(client: &Client, base_url: &str, admin_token: &str, reading_id: &Uuid) -> Result<MintResponse> {
    let response = client
        .post(&format!("{}/api/admin/meters/mint-from-reading", base_url))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&serde_json::json!({
            "reading_id": reading_id
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response)
}

async fn setup_rec_authority(client: &Client, base_url: &str) -> Result<String> {
    let email = "rec_authority@example.com";
    let password = "RecAuth123!@#";
    
    // Register REC authority (assuming this creates user with REC role)
    client
        .post(&format!("{}/api/auth/register", base_url))
        .json(&serde_json::json!({
            "email": email,
            "password": password,
            "name": "REC Authority"
        }))
        .send()
        .await?
        .error_for_status()?;

    // Login
    let login_response: LoginResponse = client
        .post(&format!("{}/api/auth/login", base_url))
        .json(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(login_response.access_token)
}

async fn issue_erc_certificate(client: &Client, base_url: &str, rec_token: &str, request: ERCIssueRequest) -> Result<ERCResponse> {
    let response = client
        .post(&format!("{}/api/erc/certificates", base_url))
        .header("Authorization", format!("Bearer {}", rec_token))
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response)
}
