#!/bin/bash
set -e

# Start Solana Validator if not running
if ! pgrep -x "solana-test-val" > /dev/null; then
    echo "Starting Solana Test Validator..."
    nohup solana-test-validator --reset --quiet > validator.log 2>&1 &
    PID=$!
    echo "Validator started with PID $PID. Log: validator.log"
    
    # Wait for validator to be ready
    echo "Waiting for validator..."
    until curl -s http://localhost:8899/health > /dev/null; do
        sleep 2
        echo -n "."
    done
    echo "Validator is ready!"
else
    echo "Solana Test Validator is already running."
fi

# Create wallets if they don't exist
mkdir -p secrets

if [ ! -f secrets/buyer.json ]; then
    echo "Creating buyer wallet..."
    solana-keygen new -o secrets/buyer.json --no-bip39-passphrase --silent
fi

if [ ! -f secrets/seller.json ]; then
    echo "Creating seller wallet..."
    solana-keygen new -o secrets/seller.json --no-bip39-passphrase --silent
fi

# Config CLI
solana config set --url localhost

# Airdrop SOL
BUYER_PUBKEY=$(solana-keygen pubkey secrets/buyer.json)
SELLER_PUBKEY=$(solana-keygen pubkey secrets/seller.json)

echo "Funding Buyer: $BUYER_PUBKEY"
solana airdrop 10 $BUYER_PUBKEY

echo "Funding Seller: $SELLER_PUBKEY"
solana airdrop 10 $SELLER_PUBKEY

# Create Token Mint (Decimals = 9)
if [ ! -f secrets/mint.json ]; then
    echo "Creating mint keypair..."
    solana-keygen new -o secrets/mint.json --no-bip39-passphrase --silent
fi

MINT_PUBKEY=$(solana-keygen pubkey secrets/mint.json)
if ! spl-token supply $MINT_PUBKEY > /dev/null 2>&1; then
    echo "Creating Energy Token Mint: $MINT_PUBKEY (Decimals: 9)"
    spl-token create-token secrets/mint.json --decimals 9
else
    echo "Mint $MINT_PUBKEY already exists."
fi

# Create ATA for Seller and Mint 1000 Tokens (Initial Supply)
echo "Minting initial supply to Seller..."

# Get or Create Seller ATA
SELLER_ATA=$(spl-token address --token $MINT_PUBKEY --owner $SELLER_PUBKEY --verbose | grep "Wallet" | awk '{print $2}' || true)

if [ -z "$SELLER_ATA" ] || ! solana account $SELLER_ATA >/dev/null 2>&1; then
    echo "Creating Seller ATA..."
    spl-token create-account $MINT_PUBKEY --owner $SELLER_PUBKEY --fee-payer ~/.config/solana/id.json || true
    # Always refresh SELLER_ATA (Use strict flag ordering: --verbose first)
    SELLER_ATA=$(spl-token address --verbose --token $MINT_PUBKEY --owner $SELLER_PUBKEY | grep "Associated token address" | awk '{print $4}')
else
    echo "Seller ATA already exists: $SELLER_ATA"
    # Recover ATA address if it existed
    SELLER_ATA=$(spl-token address --verbose --token $MINT_PUBKEY --owner $SELLER_PUBKEY | grep "Associated token address" | awk '{print $4}')
fi

echo "Minting to ATA: $SELLER_ATA"
spl-token mint $MINT_PUBKEY 1000 $SELLER_ATA --fee-payer ~/.config/solana/id.json

echo "Mint Address: $MINT_PUBKEY"
echo "Exporting MINT to .env..."
# Update or Append to .env
if grep -q "ENERGY_TOKEN_MINT=" ../.env; then
  sed -i '' "s/ENERGY_TOKEN_MINT=.*/ENERGY_TOKEN_MINT=$MINT_PUBKEY/" ../.env
else
  echo "ENERGY_TOKEN_MINT=$MINT_PUBKEY" >> ../.env
fi

echo "Environment Setup Complete!"
