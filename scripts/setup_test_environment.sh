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

echo "Environment Setup Complete!"
